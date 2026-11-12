#!/usr/bin/env python3
"""Generate explicit version-1 TOML. Python 3 standard library only.

--nodes counts routers + workstations + servers. One server per LAN.
Each workstation requires its local server: the mission is explicit too.
"""
import argparse
from pathlib import Path


def generate(nodes: int, routers: int, seed: int = 1, ticks: int = 300) -> str:
    if not 1 <= routers <= 1000 or nodes < 3 * routers:
        raise ValueError("Use 1..1000 routers and at least 3 nodes per router")
    lines = ["version = 1", "routers = ["]
    lines += [f"  {{ id = {r} }}," for r in range(1, routers + 1)]
    lines += ["]", "lans = ["]
    lines += [f"  {{ id = {r}, router = {r} }}," for r in range(1, routers + 1)]
    lines += ["]", "workstations = ["]
    # IDs use disjoint ranges: routers, then servers, then workstations.
    hosts = nodes - 2 * routers
    services = []
    for i in range(hosts):
        router = i % routers + 1
        host = 2 * routers + i + 1
        infected = ", compromised = true" if i == 0 else ""
        lines.append(f"  {{ id = {host}, lan = {router}{infected} }},")
        services.append(f"  {{ client = {host}, server = {routers + router}, weight = 1 }},")
    lines += ["]", "servers = ["]
    lines += [f"  {{ id = {routers+r}, lan = {r} }}," for r in range(1, routers + 1)]
    lines += ["]", "connections = ["]
    # A star keeps routes short; no duplicate edge on two-router networks.
    lines += [f"  {{ from = 1, to = {r} }}," for r in range(2, routers + 1)]
    lines += ["]", "", "[simulation]", f"seed = {seed}", f"tick_limit = {ticks}", "tick_ms = 0",
              "", "[mission]", "services = [", *services, "]", ""]
    return "\n".join(lines)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--nodes", type=int, required=True)
    parser.add_argument("--routers", type=int, required=True)
    parser.add_argument("--seed", type=int, default=1)
    parser.add_argument("--ticks", type=int, default=300)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        text = generate(args.nodes, args.routers, args.seed, args.ticks)
    except ValueError as error:
        parser.error(str(error))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(text)
    print(f"Wrote {args.nodes} nodes to {args.output}")
