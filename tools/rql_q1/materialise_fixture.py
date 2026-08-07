#!/usr/bin/env python3
"""Materialise RQL-Q1 corpus fixtures from generator_id + seed + params.

Deterministic only. No product I/O. See spec/rql/qualification/corpus-v1/generators/.
"""
from __future__ import annotations

import argparse
import json
import sys
from typing import Any


class SplitMix64:
    def __init__(self, seed: int) -> None:
        self.state = seed & 0xFFFFFFFFFFFFFFFF

    def next_u64(self) -> int:
        self.state = (self.state + 0x9E3779B97F4A7C15) & 0xFFFFFFFFFFFFFFFF
        z = self.state
        z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9 & 0xFFFFFFFFFFFFFFFF
        z = (z ^ (z >> 27)) * 0x94D049BB133111EB & 0xFFFFFFFFFFFFFFFF
        return z ^ (z >> 31)

    def next_u32(self) -> int:
        return self.next_u64() & 0xFFFFFFFF

    def pick(self, items: list[Any]) -> Any:
        return items[self.next_u32() % len(items)]


def iso_from_base(base: str, minutes: int) -> str:
    # Logical clock only — not calendar-correct for month rollover; fine for fixtures.
    return f"{base}+{minutes}m"


def gen_commerce_orders(seed: int, params: dict) -> dict:
    n = int(params.get("n_orders", 32))
    n_cust = int(params.get("n_customers", 16))
    regions = params.get("regions", ["us", "eu", "apac"])
    statuses = params.get("statuses", ["paid", "open", "cancelled", "shipped"])
    rng = SplitMix64(seed)
    orders = []
    for i in range(n):
        doc: dict[str, Any] = {
            "_key": f"o-{i:04d}",
            "status": statuses[i % len(statuses)],
            "region": regions[i % len(regions)],
            "created_at": iso_from_base("2024-01-01T00:00:00Z", i),
            "name": f"Order {i}",
        }
        if i % 11 == 0:
            doc["amount"] = "NaN"
        else:
            doc["amount"] = 10 + (rng.next_u32() % 990)
        if i % 7 != 0:
            doc["customer"] = {"id": f"c-{(i % n_cust):04d}"}
        if i % 13 == 0:
            doc["deleted_at"] = None
        elif i % 17 == 0:
            pass  # omit
        if i % 5 == 0:
            doc["notes"] = None
        else:
            doc["notes"] = f"note-{i}"
        if i % 9 == 0:
            doc["tags"] = []
        elif i % 10 == 0:
            doc["tags"] = ["a", "a"]
        else:
            doc["tags"] = ["sku", doc["region"]]
        orders.append(doc)
    return {"orders": orders}


def gen_commerce_products(seed: int, params: dict) -> dict:
    n = int(params.get("n_products", 48))
    cats = params.get("categories", ["widget", "gadget", "supply"])
    rng = SplitMix64(seed)
    colors = ["red", "blue", "green"]
    products = []
    for i in range(n):
        doc: dict[str, Any] = {
            "_key": f"p-{i:04d}",
            "sku": f"SKU-{i:04d}",
            "category": cats[i % len(cats)],
            "price_cents": 100 + (rng.next_u32() % 9900),
            "active": i % 5 != 0,
        }
        if i % 8 != 0:
            doc["attrs"] = {"color": rng.pick(colors)}
        products.append(doc)
    return {"products": products}


def gen_commerce_customers(seed: int, params: dict) -> dict:
    n = int(params.get("n_customers", 16))
    tiers = ["bronze", "silver", "gold"]
    regions = ["us", "eu", "apac"]
    customers = []
    for i in range(n):
        customers.append(
            {
                "_key": f"c-{i:04d}",
                "email": f"user{i}@example.test",
                "tier": tiers[i % 3],
                "region": regions[i % 3],
            }
        )
    return {"customers": customers}


def gen_commerce_line_items(seed: int, params: dict) -> dict:
    n_orders = int(params.get("n_orders", 32))
    n_products = int(params.get("n_products", 48))
    items = []
    idx = 0
    for o in range(n_orders):
        k = (o % 3) + 1
        for _ in range(k):
            items.append(
                {
                    "_key": f"li-{idx:04d}",
                    "order_id": f"o-{o:04d}",
                    "product_id": f"p-{(idx % n_products):04d}",
                    "qty": 1 + (idx % 5),
                    "unit_price_cents": 100 + idx * 7,
                }
            )
            idx += 1
    return {"line_items": items}


def gen_commerce_inventory(seed: int, params: dict) -> dict:
    n = int(params.get("n_products", 48))
    warehouses = params.get("warehouses", ["w1", "w2"])
    rng = SplitMix64(seed)
    inv = []
    for i in range(n):
        for wh in warehouses:
            on = rng.next_u32() % 500
            res = min(on, rng.next_u32() % 50)
            inv.append(
                {
                    "_key": f"inv-p-{i:04d}-{wh}",
                    "product_id": f"p-{i:04d}",
                    "warehouse": wh,
                    "qty_on_hand": on,
                    "qty_reserved": res,
                }
            )
    return {"inventory": inv}


def gen_messaging_conversations(seed: int, params: dict) -> dict:
    n = int(params.get("n_conversations", 24))
    rng = SplitMix64(seed)
    out = []
    for i in range(n):
        created = iso_from_base("2024-06-01T00:00:00Z", i * 60)
        out.append(
            {
                "_key": f"cv-{i:04d}",
                "title": f"Conversation {i}",
                "kind": "direct" if i % 4 == 0 else "group",
                "created_at": created,
                "archived": i % 11 == 0,
                "last_message_at": iso_from_base("2024-06-01T00:00:00Z", i * 60 + (rng.next_u32() % 1000)),
            }
        )
    return {"conversations": out}


def gen_messaging_messages(seed: int, params: dict) -> dict:
    n = int(params.get("n_messages", 96))
    n_cv = int(params.get("n_conversations", 24))
    n_users = int(params.get("n_users", 12))
    out = []
    for i in range(n):
        doc: dict[str, Any] = {
            "_key": f"m-{i:04d}",
            "conversation_id": f"cv-{(i % n_cv):04d}",
            "sender_id": f"u-{(i % n_users):04d}",
            "sent_at": iso_from_base("2024-06-01T00:00:00Z", i),
            "edited": i % 13 == 0,
        }
        if i % 7 == 0:
            doc["body"] = None
        else:
            doc["body"] = f"msg body {i}"
        if i % 3 == 0:
            doc["read_at"] = iso_from_base("2024-06-01T00:00:00Z", i + 5)
        elif i % 5 == 0:
            pass  # missing
        else:
            doc["read_at"] = None
        if i % 9 == 0:
            doc["attachments"] = [{"type": "image", "name": "a.jpg"}]
        else:
            doc["attachments"] = []
        out.append(doc)
    return {"messages": out}


def gen_messaging_participants(seed: int, params: dict) -> dict:
    n_cv = int(params.get("n_conversations", 24))
    n_users = int(params.get("n_users", 12))
    out = []
    for c in range(n_cv):
        count = 2 + (c % 3)
        for j in range(count):
            u = (c + j) % n_users
            out.append(
                {
                    "_key": f"pt-cv-{c:04d}-u-{u:04d}",
                    "conversation_id": f"cv-{c:04d}",
                    "user_id": f"u-{u:04d}",
                    "role": "owner" if j == 0 else "member",
                    "muted": (c + j) % 6 == 0,
                    "joined_at": iso_from_base("2024-06-01T00:00:00Z", c * 60 + j),
                }
            )
    return {"participants": out}



def gen_directory_entries(seed: int, params: dict) -> dict:
    n = int(params.get("n_entries", 40))
    n_cats = int(params.get("n_categories", 8))
    n_locs = int(params.get("n_locations", 12))
    kinds = params.get("kinds", ["person", "org"])
    rng = SplitMix64(seed)
    out = []
    for i in range(n):
        doc: dict[str, Any] = {
            "_key": f"e-{i:04d}",
            "name": f"Entry {i}",
            "kind": kinds[i % len(kinds)],
            "category_id": f"cat-{(i % n_cats):04d}",
            "active": i % 6 != 0,
            "discovered_at": iso_from_base("2024-03-01T00:00:00Z", i * 30),
        }
        if i % 5 != 0:
            doc["location_id"] = f"loc-{(i % n_locs):04d}"
        # email: null every 4th; missing every 11th; else present
        if i % 11 == 0:
            pass
        elif i % 4 == 0:
            doc["email"] = None
        else:
            doc["email"] = f"entry{i}@dir.example.test"
        if i % 7 == 0:
            doc["tags"] = []
        elif i % 9 == 0:
            doc["tags"] = ["vip", "vip"]
        else:
            doc["tags"] = ["dir", kinds[i % len(kinds)]]
        if i % 8 != 0:
            doc["attrs"] = {"region": rng.pick(["us", "eu", "apac"]), "score": i % 100}
        out.append(doc)
    return {"entries": out}


def gen_directory_categories(seed: int, params: dict) -> dict:
    n = int(params.get("n_categories", 8))
    parents = params.get("parent_slugs", ["people", "orgs", "places"])
    out = []
    for i in range(n):
        out.append(
            {
                "_key": f"cat-{i:04d}",
                "slug": f"cat-slug-{i}",
                "title": f"Category {i}",
                "parent": parents[i % len(parents)],
                "depth": i % 3,
            }
        )
    return {"categories": out}


def gen_directory_locations(seed: int, params: dict) -> dict:
    n = int(params.get("n_locations", 12))
    regions = params.get("regions", ["us", "eu", "apac"])
    out = []
    for i in range(n):
        out.append(
            {
                "_key": f"loc-{i:04d}",
                "city": f"City{i}",
                "region": regions[i % len(regions)],
                "country": regions[i % len(regions)].upper(),
                "lat": -90 + (i * 7) % 180,
                "lon": -180 + (i * 13) % 360,
            }
        )
    return {"locations": out}


def gen_telemetry_devices(seed: int, params: dict) -> dict:
    n = int(params.get("n_devices", 24))
    types = params.get("types", ["sensor", "gateway", "actuator"])
    sites = params.get("sites", ["plant-a", "plant-b", "field"])
    statuses = params.get("statuses", ["online", "offline", "degraded"])
    out = []
    for i in range(n):
        doc: dict[str, Any] = {
            "_key": f"d-{i:04d}",
            "type": types[i % len(types)],
            "site": sites[i % len(sites)],
            "status": statuses[i % len(statuses)],
            "last_seen": iso_from_base("2024-08-01T00:00:00Z", i * 15),
            "firmware": f"1.{i % 5}.{i % 10}",
        }
        if i % 5 == 0:
            doc["labels"] = []
        else:
            doc["labels"] = ["edge", sites[i % len(sites)]]
        if i % 7 == 0:
            doc["retired_at"] = None
        elif i % 11 == 0:
            pass  # omit retired_at
        out.append(doc)
    return {"devices": out}


def gen_telemetry_events(seed: int, params: dict) -> dict:
    n = int(params.get("n_events", 128))
    n_dev = int(params.get("n_devices", 24))
    sevs = params.get("severities", ["info", "warn", "error", "critical"])
    rng = SplitMix64(seed)
    out = []
    for i in range(n):
        doc: dict[str, Any] = {
            "_key": f"ev-{i:04d}",
            "device_id": f"d-{(i % n_dev):04d}",
            "ts": iso_from_base("2024-08-01T00:00:00Z", i),
            "severity": sevs[i % len(sevs)],
            "metric": rng.pick(["temp_c", "humidity", "voltage", "rpm"]),
            "value": (rng.next_u32() % 1000) / 10.0,
        }
        if i % 9 == 0:
            doc["value"] = "NaN"  # wrong type
        if i % 6 == 0:
            doc["payload"] = None
        elif i % 8 == 0:
            pass  # missing payload
        else:
            doc["payload"] = {"raw": i, "unit": "u"}
        if i % 10 == 0:
            doc["tags"] = []
        else:
            doc["tags"] = [doc["metric"], sevs[i % len(sevs)]]
        out.append(doc)
    return {"events": out}


def gen_telemetry_metrics(seed: int, params: dict) -> dict:
    n = int(params.get("n_metrics", 64))
    n_dev = int(params.get("n_devices", 24))
    names = params.get("metric_names", ["temp_c", "humidity", "voltage"])
    out = []
    for i in range(n):
        out.append(
            {
                "_key": f"mt-{i:04d}",
                "device_id": f"d-{(i % n_dev):04d}",
                "metric_name": names[i % len(names)],
                "value": float(i % 100),
                "recorded_at": iso_from_base("2024-08-01T00:00:00Z", i * 5),
            }
        )
    return {"metrics": out}


def gen_pm_projects(seed: int, params: dict) -> dict:
    n = int(params.get("n_projects", 20))
    statuses = params.get("statuses", ["active", "on_hold", "done", "archived"])
    owners = params.get("owners", [f"u-{i:04d}" for i in range(8)])
    out = []
    for i in range(n):
        doc: dict[str, Any] = {
            "_key": f"prj-{i:04d}",
            "title": f"Project {i}",
            "status": statuses[i % len(statuses)],
            "owner_id": owners[i % len(owners)],
            "created_at": iso_from_base("2024-02-01T00:00:00Z", i * 120),
            "priority": 1 + (i % 5),
        }
        if i % 4 == 0:
            doc["description"] = None
        elif i % 7 == 0:
            pass
        else:
            doc["description"] = f"Desc {i}"
        if i % 5 != 0:
            doc["labels"] = ["pm", statuses[i % len(statuses)]]
        else:
            doc["labels"] = []
        out.append(doc)
    return {"projects": out}


def gen_pm_tasks(seed: int, params: dict) -> dict:
    n = int(params.get("n_tasks", 80))
    n_prj = int(params.get("n_projects", 20))
    n_users = int(params.get("n_users", 8))
    statuses = params.get("statuses", ["todo", "doing", "in_review", "done"])
    out = []
    for i in range(n):
        doc: dict[str, Any] = {
            "_key": f"tsk-{i:04d}",
            "project_id": f"prj-{(i % n_prj):04d}",
            "title": f"Task {i}",
            "status": statuses[i % len(statuses)],
            "assignee_id": f"u-{(i % n_users):04d}",
            "updated_at": iso_from_base("2024-02-01T00:00:00Z", i * 10),
            "estimate_points": 1 + (i % 8),
        }
        if i % 6 == 0:
            doc["due_at"] = None
        elif i % 9 == 0:
            pass
        else:
            doc["due_at"] = iso_from_base("2024-03-01T00:00:00Z", i * 20)
        if i % 11 == 0:
            doc["blocked_by"] = []
        else:
            doc["blocked_by"] = [f"tsk-{(i - 1) % n:04d}"] if i > 0 and i % 3 == 0 else []
        out.append(doc)
    return {"tasks": out}


def gen_pm_revisions(seed: int, params: dict) -> dict:
    n = int(params.get("n_revisions", 48))
    n_prj = int(params.get("n_projects", 20))
    n_users = int(params.get("n_users", 8))
    out = []
    for i in range(n):
        out.append(
            {
                "_key": f"rev-{i:04d}",
                "project_id": f"prj-{(i % n_prj):04d}",
                "rev_no": (i // n_prj) + 1,
                "author_id": f"u-{(i % n_users):04d}",
                "summary": f"Revision {i}",
                "committed_at": iso_from_base("2024-02-15T00:00:00Z", i * 30),
                "bytes": 100 + i * 17,
            }
        )
    return {"revisions": out}


def gen_pm_memberships(seed: int, params: dict) -> dict:
    n_prj = int(params.get("n_projects", 20))
    n_users = int(params.get("n_users", 8))
    roles = params.get("roles", ["owner", "editor", "viewer"])
    out = []
    for p in range(n_prj):
        count = 2 + (p % 3)
        for j in range(count):
            u = (p + j) % n_users
            out.append(
                {
                    "_key": f"mb-prj-{p:04d}-u-{u:04d}",
                    "project_id": f"prj-{p:04d}",
                    "user_id": f"u-{u:04d}",
                    "role": roles[0] if j == 0 else roles[1 + (j % (len(roles) - 1))],
                    "joined_at": iso_from_base("2024-02-01T00:00:00Z", p * 60 + j),
                }
            )
    return {"memberships": out}



GENERATORS = {
    "commerce.orders_v1": gen_commerce_orders,
    "commerce.products_v1": gen_commerce_products,
    "commerce.customers_v1": gen_commerce_customers,
    "commerce.line_items_v1": gen_commerce_line_items,
    "commerce.inventory_v1": gen_commerce_inventory,
    "messaging.conversations_v1": gen_messaging_conversations,
    "messaging.messages_v1": gen_messaging_messages,
    "messaging.participants_v1": gen_messaging_participants,
    "directory.entries_v1": gen_directory_entries,
    "directory.categories_v1": gen_directory_categories,
    "directory.locations_v1": gen_directory_locations,
    "telemetry.devices_v1": gen_telemetry_devices,
    "telemetry.events_v1": gen_telemetry_events,
    "telemetry.metrics_v1": gen_telemetry_metrics,
    "project_management.projects_v1": gen_pm_projects,
    "project_management.tasks_v1": gen_pm_tasks,
    "project_management.revisions_v1": gen_pm_revisions,
    "project_management.memberships_v1": gen_pm_memberships,
}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--generator", required=True)
    ap.add_argument("--seed", type=int, required=True)
    ap.add_argument("--params", default="{}")
    args = ap.parse_args()
    if args.generator not in GENERATORS:
        print(f"unknown generator: {args.generator}", file=sys.stderr)
        print("known:", ", ".join(sorted(GENERATORS)), file=sys.stderr)
        return 2
    params = json.loads(args.params)
    collections = GENERATORS[args.generator](args.seed, params)
    json.dump(
        {
            "generator_id": args.generator,
            "seed": args.seed,
            "params": params,
            "collections": collections,
        },
        sys.stdout,
        indent=2,
        sort_keys=True,
    )
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
