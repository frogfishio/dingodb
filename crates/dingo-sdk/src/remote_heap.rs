//! Qualified remote heap connection (`HEAP_SPEC` §7.1 / §30.9 / HP-007 residual).
//!
//! `Dingo::connect_heap` performs TLS 1.3 + HeapKey handshake
//! (`hello` → `heap_challenge` → `heap_auth` → `welcome`) and returns a
//! [`RemoteHeap`] session bound to one `HeapId`. Active process ops 1–3
//! (ping / live / ready) are available; collection data ops remain reserved
//! until §32.4 activation.

use crate::error::Error;
use crate::remote::{parse_dingo_url, DEFAULT_PORT};
use crate::tls::{client_connect, IoStream, TlsClientOptions};
use dingo_client::{
    b64u_decode, b64u_encode, read_frame, write_json_frame, Handshake, HeapAuth, HeapChallenge,
    HeapReject, HeapWelcome, FEATURE_HEAP_KEY_V1, HANDSHAKE_MAX_FRAME_BYTES, HEAP_AUTH_MAX_BYTES,
};
use dingo_heap::{
    build_holder_proof, inspect_certificate, HeapId, VerifiedCertificate, HEAP_PROFILE,
};
use serde_json::{Map, Value};
use std::net::{TcpStream, ToSocketAddrs};
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Errors from credential construction (local only; never contact a server).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialError {
    /// Certificate COSE failed structural inspection.
    MalformedCertificate(String),
    /// Holder public key does not match the certificate claim.
    HolderKeyMismatch,
    /// Signer failed to produce a proof signature.
    SignFailed(String),
}

impl std::fmt::Display for CredentialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedCertificate(m) => write!(f, "malformed certificate: {m}"),
            Self::HolderKeyMismatch => write!(f, "holder public key mismatch"),
            Self::SignFailed(m) => write!(f, "holder proof sign failed: {m}"),
        }
    }
}

impl std::error::Error for CredentialError {}

impl From<CredentialError> for Error {
    fn from(e: CredentialError) -> Self {
        Error::AuthenticationFailed(e.to_string())
    }
}

/// Signs the COSE `Sig_structure` for a holder proof without exporting secrets.
pub trait HolderSigner: Send + Sync {
    /// Ed25519 verifying key bytes claimed by the certificate.
    fn public_key(&self) -> [u8; 32];
    /// Sign the holder-proof `Sig_structure` message.
    fn sign_holder_proof(&self, message: &[u8]) -> Result<[u8; 64], CredentialError>;
}

/// Local HeapKey credential: certificate bytes + holder signer.
///
/// Not `Debug` / `Serialize` / `Deserialize`. Clone only by cloning the signer
/// handle. Master signature is verified by the **server**, not here.
pub struct HeapCredential {
    certificate_cose: Vec<u8>,
    inspected: VerifiedCertificate,
    signer: Arc<dyn HolderSigner>,
}

impl HeapCredential {
    /// Bind `certificate_cose` to `signer` after structural inspect.
    ///
    /// Rejects a signer whose public key differs from the certificate claim.
    /// Does not contact a server and does not verify the master signature.
    pub fn new(
        certificate_cose: &[u8],
        signer: Arc<dyn HolderSigner>,
    ) -> Result<Self, CredentialError> {
        let inspected = inspect_certificate(certificate_cose).map_err(|e| {
            CredentialError::MalformedCertificate(e.to_string())
        })?;
        if inspected.holder_public_key != signer.public_key() {
            return Err(CredentialError::HolderKeyMismatch);
        }
        Ok(Self {
            certificate_cose: certificate_cose.to_vec(),
            inspected,
            signer,
        })
    }

    /// Heap id claimed by the certificate (informational until server welcomes).
    pub fn heap_id(&self) -> HeapId {
        self.inspected.heap_id
    }

    /// Certificate fingerprint (SHA-256 of COSE bytes).
    pub fn certificate_fingerprint(&self) -> [u8; 32] {
        self.inspected.fingerprint
    }

    /// Explicit clone via shared signer handle.
    pub fn clone_credential(&self) -> Self {
        Self {
            certificate_cose: self.certificate_cose.clone(),
            inspected: self.inspected.clone(),
            signer: Arc::clone(&self.signer),
        }
    }
}

/// In-memory Ed25519 holder key (tests / local dev only).
///
/// Gated behind feature `dangerous-key-export`. Does not expose a raw secret
/// export method; construction from a 32-byte seed is the gated surface.
#[cfg(feature = "dangerous-key-export")]
pub struct InMemoryHolderKey {
    sk: ed25519_dalek::SigningKey,
}

#[cfg(feature = "dangerous-key-export")]
impl InMemoryHolderKey {
    /// Construct from a 32-byte seed (reference / test only).
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            sk: ed25519_dalek::SigningKey::from_bytes(&seed),
        }
    }
}

#[cfg(feature = "dangerous-key-export")]
impl HolderSigner for InMemoryHolderKey {
    fn public_key(&self) -> [u8; 32] {
        self.sk.verifying_key().to_bytes()
    }

    fn sign_holder_proof(&self, message: &[u8]) -> Result<[u8; 64], CredentialError> {
        use ed25519_dalek::Signer;
        Ok(self.sk.sign(message).to_bytes())
    }
}

/// Options for [`crate::Dingo::connect_heap`].
///
/// TLS is mandatory. There is no token, role, username, diagnostic-line,
/// plaintext, or caller-supplied heap-id field.
pub struct RemoteHeapOptions {
    /// TLS client identity / trust.
    pub tls: TlsClientOptions,
    /// HeapKey credential.
    pub credential: HeapCredential,
    /// Optional human name checked after authority succeeds.
    pub expected_heap_name: Option<String>,
    /// Per-attempt TCP connect timeout.
    pub connect_timeout: Duration,
    /// Read/write timeout on the established stream.
    pub request_timeout: Duration,
    /// Total connect attempts (at least 1).
    pub max_connect_attempts: NonZeroU32,
    /// Backoff between connect attempts.
    pub retry_backoff: Duration,
    /// Override security-time for proof `created_at` (tests).
    pub now_unix_s: Option<u64>,
}

impl RemoteHeapOptions {
    /// Defaults: no expected name, 5s connect, 30s request, 3 attempts, 50ms backoff.
    pub fn new(tls: TlsClientOptions, credential: HeapCredential) -> Self {
        Self {
            tls,
            credential,
            expected_heap_name: None,
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_connect_attempts: NonZeroU32::new(3).expect("3"),
            retry_backoff: Duration::from_millis(50),
            now_unix_s: None,
        }
    }

    /// Expected human heap label checked after authority.
    pub fn expected_heap_name(mut self, name: impl Into<String>) -> Self {
        self.expected_heap_name = Some(name.into());
        self
    }

    /// Per-attempt TCP connect timeout.
    pub fn connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = value;
        self
    }

    /// Stream I/O timeout after connect.
    pub fn request_timeout(mut self, value: Duration) -> Self {
        self.request_timeout = value;
        self
    }

    /// Total connect attempts (including the first).
    pub fn max_connect_attempts(mut self, value: NonZeroU32) -> Self {
        self.max_connect_attempts = value;
        self
    }

    /// Sleep between failed connect attempts.
    pub fn retry_backoff(mut self, value: Duration) -> Self {
        self.retry_backoff = value;
        self
    }

    /// Pin proof `created_at` (and local time basis) for tests.
    pub fn now_unix_s(mut self, value: u64) -> Self {
        self.now_unix_s = Some(value);
        self
    }
}

/// Established qualified remote heap session (HP-007 `connect_heap`).
///
/// Bound to a single `HeapId` from the server welcome. Exposes process ops
/// 1–3 until collection data ops activate under §32.4.
pub struct RemoteHeap {
    stream: IoStream,
    welcome: HeapWelcome,
    heap_id: HeapId,
    max_frame: usize,
    next_id: AtomicU64,
}

impl RemoteHeap {
    /// Bound heap id from the server welcome.
    pub fn id(&self) -> HeapId {
        self.heap_id
    }

    /// Wire welcome object.
    pub fn welcome(&self) -> &HeapWelcome {
        &self.welcome
    }

    /// Session id from welcome (capability/session hex).
    pub fn session_id(&self) -> &str {
        &self.welcome.session_id
    }

    /// Heap profile string from welcome.
    pub fn heap_profile(&self) -> &str {
        &self.welcome.heap_profile
    }

    /// Qualified process ping (op_id = 1).
    pub fn ping(&mut self) -> Result<(), Error> {
        let result = self.call_process(1)?;
        if result.get("pong") == Some(&Value::Bool(true)) {
            Ok(())
        } else {
            Err(Error::ProtocolViolation(
                "qualified ping missing pong:true".into(),
            ))
        }
    }

    /// Qualified live probe (op_id = 2).
    pub fn live(&mut self) -> Result<bool, Error> {
        let result = self.call_process(2)?;
        Ok(result.get("live") == Some(&Value::Bool(true)))
    }

    /// Qualified ready probe (op_id = 3).
    pub fn ready(&mut self) -> Result<bool, Error> {
        let result = self.call_process(3)?;
        Ok(result.get("ready") == Some(&Value::Bool(true)))
    }

    /// Open a collection by canonical name (op_id = 105). Returns collection UUID string.
    pub fn collection_open(&mut self, name: &str) -> Result<String, Error> {
        let result = self.call_args(105, None, serde_json::json!({ "name": name }))?;
        result
            .get("collection_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::ProtocolViolation("collection_open missing collection_id".into()))
    }

    /// Put JSON under `key` in `collection_id` (op_id = 120).
    pub fn put_json(
        &mut self,
        collection_id: &str,
        key: &str,
        json: &Value,
    ) -> Result<(String, String), Error> {
        let result = self.call_args(
            120,
            Some(collection_id),
            serde_json::json!({ "key": key, "json": json }),
        )?;
        let event_id = result
            .get("event_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ProtocolViolation("put missing event_id".into()))?
            .to_string();
        let version = result
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ProtocolViolation("put missing version".into()))?
            .to_string();
        Ok((event_id, version))
    }

    /// Put opaque bytes under `key` (op_id = 121).
    pub fn put_bytes(
        &mut self,
        collection_id: &str,
        key: &str,
        bytes: &[u8],
    ) -> Result<(String, String), Error> {
        let result = self.call_args(
            121,
            Some(collection_id),
            serde_json::json!({
                "key": key,
                "bytes_b64": dingo_client::b64u_encode(bytes),
            }),
        )?;
        let event_id = result
            .get("event_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ProtocolViolation("put_bytes missing event_id".into()))?
            .to_string();
        let version = result
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::ProtocolViolation("put_bytes missing version".into()))?
            .to_string();
        Ok((event_id, version))
    }

    /// Get JSON for `key` (op_id = 111). `None` when not found.
    pub fn get_json(
        &mut self,
        collection_id: &str,
        key: &str,
    ) -> Result<Option<Value>, Error> {
        let result =
            self.call_args(111, Some(collection_id), serde_json::json!({ "key": key }))?;
        match result.get("found") {
            Some(Value::Bool(false)) => Ok(None),
            Some(Value::Bool(true)) => Ok(result.get("json").cloned()),
            _ => Err(Error::ProtocolViolation("get missing found".into())),
        }
    }

    /// Get opaque bytes for `key` (op_id = 112).
    pub fn get_bytes(
        &mut self,
        collection_id: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, Error> {
        let result =
            self.call_args(112, Some(collection_id), serde_json::json!({ "key": key }))?;
        match result.get("found") {
            Some(Value::Bool(false)) => Ok(None),
            Some(Value::Bool(true)) => {
                let b64 = result
                    .get("bytes_b64")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::ProtocolViolation("get_bytes missing bytes_b64".into()))?;
                dingo_client::b64u_decode(b64)
                    .map(Some)
                    .map_err(|e| Error::ProtocolViolation(format!("bytes_b64: {e}")))
            }
            _ => Err(Error::ProtocolViolation("get_bytes missing found".into())),
        }
    }

    /// Delete `key` (op_id = 122). Returns whether a value was present.
    pub fn delete(&mut self, collection_id: &str, key: &str) -> Result<bool, Error> {
        let result =
            self.call_args(122, Some(collection_id), serde_json::json!({ "key": key }))?;
        Ok(result.get("removed") == Some(&Value::Bool(true)))
    }

    fn call_process(&mut self, op_id: u16) -> Result<Value, Error> {
        self.call_args(op_id, None, serde_json::json!({}))
    }

    fn call_args(
        &mut self,
        op_id: u16,
        collection_id: Option<&str>,
        args: Value,
    ) -> Result<Value, Error> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut req = serde_json::json!({
            "v": 1,
            "id": id,
            "op_id": op_id,
            "args": args,
        });
        if let Some(cid) = collection_id {
            req.as_object_mut()
                .unwrap()
                .insert("collection_id".into(), Value::String(cid.into()));
        }
        write_json_frame(&mut self.stream, &req).map_err(Error::from)?;
        let resp_bytes = read_frame(&mut self.stream, self.max_frame)
            .map_err(Error::from)?
            .ok_or_else(|| Error::ProtocolViolation("connection closed during RPC".into()))?;
        let resp: Value = serde_json::from_slice(&resp_bytes)
            .map_err(|e| Error::ProtocolViolation(format!("rpc response: {e}")))?;
        if resp.get("ok") != Some(&Value::Bool(true)) {
            let code = resp
                .pointer("/error/code")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            return Err(Error::Remote {
                code: code.into(),
                message: format!("qualified op {op_id} failed"),
            });
        }
        Ok(resp
            .get("result")
            .cloned()
            .unwrap_or(Value::Object(Map::new())))
    }
}

/// Connect to a qualified heap listener (`dingo://host:port[/label]`).
///
/// The URL path label is an expected human name only when
/// [`RemoteHeapOptions::expected_heap_name`] is not set; when both are set the
/// option wins. Authority comes solely from the credential certificate.
pub fn connect_heap(
    url: impl AsRef<str>,
    options: RemoteHeapOptions,
) -> Result<RemoteHeap, Error> {
    let url = url.as_ref();
    let parsed = parse_dingo_url(url)?;
    if parsed.seeds.is_empty() {
        return Err(Error::ValidationMsg("empty dingo:// URL".into()));
    }
    let expected_name = options
        .expected_heap_name
        .clone()
        .or_else(|| parsed.label.clone().filter(|s| !s.is_empty()));

    let mut last_err: Option<Error> = None;
    for attempt in 0..options.max_connect_attempts.get() {
        if attempt > 0 {
            thread::sleep(options.retry_backoff);
        }
        for hostport in &parsed.seeds {
            match connect_heap_once(hostport, &options, expected_name.as_deref()) {
                Ok(remote) => return Ok(remote),
                Err(e) => last_err = Some(e),
            }
        }
    }
    Err(last_err.unwrap_or_else(|| Error::Internal("connect_heap failed".into())))
}

fn connect_heap_once(
    hostport: &str,
    options: &RemoteHeapOptions,
    expected_heap_name: Option<&str>,
) -> Result<RemoteHeap, Error> {
    let addr = resolve_hostport(hostport)?;
    let tcp = TcpStream::connect_timeout(&addr, options.connect_timeout).map_err(|e| {
        Error::from_io(std::io::Error::new(
            e.kind(),
            format!("connect {hostport}: {e}"),
        ))
    })?;
    tcp.set_read_timeout(Some(options.request_timeout))
        .map_err(Error::from_io)?;
    tcp.set_write_timeout(Some(options.request_timeout))
        .map_err(Error::from_io)?;

    let (mut stream, _) = client_connect(tcp, &options.tls)?;
    if !stream.is_tls() {
        return Err(Error::ProtocolViolation(
            "connect_heap requires TLS 1.3".into(),
        ));
    }
    let exporter = stream.export_channel_binding()?;

    // hello with heap-key-v1
    let mut hello = Handshake::hello();
    let mut feats = hello.features.take().unwrap_or_default();
    if !feats.iter().any(|f| f == FEATURE_HEAP_KEY_V1) {
        feats.push(FEATURE_HEAP_KEY_V1.into());
    }
    hello.features = Some(feats);
    write_json_frame(&mut stream, &hello).map_err(Error::from)?;

    let challenge_bytes = read_frame(&mut stream, HANDSHAKE_MAX_FRAME_BYTES)
        .map_err(Error::from)?
        .ok_or_else(|| {
            Error::ProtocolViolation("connection closed before heap_challenge".into())
        })?;
    // Uniform reject before challenge is a protocol error shape; parse either.
    if let Ok(reject) = serde_json::from_slice::<HeapReject>(&challenge_bytes) {
        if reject.msg == "reject" {
            return Err(Error::Remote {
                code: reject.code,
                message: "heap unavailable during handshake".into(),
            });
        }
    }
    let challenge: HeapChallenge = serde_json::from_slice(&challenge_bytes).map_err(|e| {
        Error::ProtocolViolation(format!("heap_challenge decode: {e}"))
    })?;
    if challenge.msg != "heap_challenge" {
        return Err(Error::ProtocolViolation(format!(
            "expected heap_challenge, got {}",
            challenge.msg
        )));
    }
    let nonce_vec = b64u_decode(&challenge.server_nonce_b64u)
        .map_err(|e| Error::ProtocolViolation(format!("nonce b64u: {e}")))?;
    if nonce_vec.len() != 32 {
        return Err(Error::ProtocolViolation("nonce must be 32 bytes".into()));
    }
    let mut nonce = [0u8; 32];
    nonce.copy_from_slice(&nonce_vec);

    let now_unix = options.now_unix_s.unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    });
    let mut proof_id = [0u8; 16];
    getrandom::fill(&mut proof_id)
        .map_err(|_| Error::Internal("getrandom failed for proof_id".into()))?;

    let cert = &options.credential.inspected;
    let signer = Arc::clone(&options.credential.signer);
    let proof = build_holder_proof(cert, &nonce, &exporter, proof_id, now_unix, |msg| {
        signer.sign_holder_proof(msg).map_err(|_| {
            dingo_heap::HeapError::unavailable(
                dingo_heap::HeapUnavailableCause::MalformedOrBadSignature,
            )
        })
    })
    .map_err(|e| Error::AuthenticationFailed(format!("holder proof: {e}")))?;

    let auth = HeapAuth {
        v: 1,
        msg: "heap_auth".into(),
        heap_id: cert.heap_id.to_string(),
        certificate_b64u: b64u_encode(&options.credential.certificate_cose),
        holder_proof_b64u: b64u_encode(&proof),
        expected_heap_name: expected_heap_name.map(|s| s.to_string()),
    };
    write_json_frame(&mut stream, &auth).map_err(Error::from)?;

    let welcome_bytes = read_frame(&mut stream, HEAP_AUTH_MAX_BYTES)
        .map_err(Error::from)?
        .ok_or_else(|| Error::ProtocolViolation("connection closed before welcome".into()))?;
    if let Ok(reject) = serde_json::from_slice::<HeapReject>(&welcome_bytes) {
        if reject.msg == "reject" {
            return Err(Error::Remote {
                code: reject.code,
                message: "heap unavailable after heap_auth".into(),
            });
        }
    }
    let welcome: HeapWelcome = serde_json::from_slice(&welcome_bytes)
        .map_err(|e| Error::ProtocolViolation(format!("welcome decode: {e}")))?;
    if welcome.msg != "welcome" {
        return Err(Error::ProtocolViolation(format!(
            "expected welcome, got {}",
            welcome.msg
        )));
    }
    if welcome.heap_profile != HEAP_PROFILE {
        return Err(Error::ProtocolViolation(format!(
            "unexpected heap_profile {}",
            welcome.heap_profile
        )));
    }
    let heap_id: HeapId = welcome
        .heap_id
        .parse()
        .map_err(|e| Error::ProtocolViolation(format!("welcome heap_id: {e}")))?;
    if heap_id != cert.heap_id {
        return Err(Error::ProtocolViolation(
            "welcome heap_id mismatches credential".into(),
        ));
    }

    Ok(RemoteHeap {
        stream,
        welcome,
        heap_id,
        max_frame: dingo_client::DEFAULT_MAX_FRAME_BYTES,
        next_id: AtomicU64::new(1),
    })
}

fn resolve_hostport(hostport: &str) -> Result<std::net::SocketAddr, Error> {
    let with_port = if hostport.contains(':') {
        hostport.to_string()
    } else {
        format!("{hostport}:{DEFAULT_PORT}")
    };
    with_port
        .to_socket_addrs()
        .map_err(|e| Error::ValidationMsg(format!("resolve {with_port}: {e}")))?
        .next()
        .ok_or_else(|| Error::ValidationMsg(format!("no addresses for {with_port}")))
}