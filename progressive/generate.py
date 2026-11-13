#!/usr/bin/env python3
"""Reproduce progressive Vanguard networks, including the 100k extension."""
from pathlib import Path
import random

LEVELS = [
    # total nodes, LANs, initial infections, deadline
    (10, 1, 1, 120),
    (100, 2, 2, 180),
    (250, 3, 4, 240),
    (500, 5, 8, 300),
    (1000, 8, 16, 360),
    (1500, 12, 30, 420),
    (2500, 16, 48, 480),
    (5000, 24, 96, 540),
    (7500, 32, 160, 600),
    (10000, 40, 240, 660),
    (100000, 400, 2400, 720),
]


def generate(level, total, count, infected_count, deadline):
    seed = 20261112 + level * 997
    rng = random.Random(seed)
    routers = rng.sample(range(1, 997), count)
    lans = rng.sample(range(2000, 9000), count)
    servers_per_lan = 1 if level <= 2 else 2
    workstation_count = total - count * (1 + servers_per_lan)
    # Uneven LANs; apportion exactly, including at least one workstation per LAN.
    weights = [[5, 3, 2, 1][i % 4] for i in range(count)]
    remaining = workstation_count - count
    sizes = [1 + remaining * weight // sum(weights) for weight in weights]
    order = sorted(range(count), key=lambda i: (-(remaining * weights[i] % sum(weights)), i))
    for i in order[:workstation_count - sum(sizes)]:
        sizes[i] += 1
    assert sum(sizes) == workstation_count
    assert max(size + servers_per_lan + 1 for size in sizes) <= 500

    ids = list(range(10001, 10001 + workstation_count * 7, 7))
    rng.shuffle(ids)
    hosts, cursor = [], 0
    for size in sizes:
        hosts.append(sorted(ids[cursor:cursor + size]))
        cursor += size
    # Preserve the original ten fixtures byte-for-byte. The extension's much
    # larger workstation ID range needs a separate, non-overlapping server band.
    server_base = 2000003 if total >= 100000 else 200003
    servers = [[server_base + (i * servers_per_lan + j) * 11
                for j in range(servers_per_lan)] for i in range(count)]

    # Distribute incidents, then add simultaneous sources on harder levels.
    # Mixed sorted positions discourage relying on ascending or reverse ID sweeps.
    infected, lan_order = set(), list(range(count))
    rng.shuffle(lan_order)
    for index in range(infected_count):
        lan_index = lan_order[index % count]
        members = hosts[lan_index]
        positions = [0, len(members) - 1, len(members) // 2,
                     len(members) // 4, 3 * len(members) // 4]
        position = positions[(index + index // count) % len(positions)]
        while members[position] in infected:
            position = (position + 1) % len(members)
        infected.add(members[position])
    assert len(infected) == infected_count

    edges = set()
    def link(a, b):
        if a != b:
            edges.add(tuple(sorted((routers[a], routers[b]))))

    sites = 4 if count >= 16 else 3 if count == 12 else 1
    per_site = count // sites
    if level <= 3:
        for i in range(count - 1):
            link(i, i + 1)
    else:
        for site in range(sites):
            base = site * per_site
            for i in range(per_site):
                link(base + i, base + (i + 1) % per_site)
            if per_site >= 4:
                link(base + 1, base + per_site - 1)
            if per_site >= 8:
                link(base + 2, base + per_site // 2)
            if sites > 1:
                link(base, ((site + 1) % sites) * per_site)
        if sites == 4:
            link(1, 2 * per_site + 1)
            if level >= 9:
                link(per_site + 2, 3 * per_site + 2)

    services = []
    for i, members in enumerate(hosts):
        site, offset = divmod(i, per_site)
        neighbour = site * per_site + (offset + 1) % per_site
        remote = ((site + 1) % sites) * per_site if sites > 1 else (i + count // 2) % count
        for index, host in enumerate(members):
            slot = index % servers_per_lan
            services.append((host, servers[i][slot], 2))
            if level >= 2:
                services.append((host, servers[neighbour][slot], 2))
            if level >= 7:
                services.append((host, servers[remote][slot], 3))

    lines = [
        f'# Vanguard progressive level {level:02}: {total:,} total nodes, {count} LANs.',
        f'# {count} routers + {workstation_count} workstations + {count * servers_per_lan} servers.',
        f'# Maximum LAN size: {max(sizes) + servers_per_lan + 1} nodes INCLUDING router and servers.',
        f'# {infected_count} initial infections; {len(edges)} links; {len(services):,} required pairs.',
        '# Standard engine fuel, memory, task slots and infection timings.',
        '# Reproduce: python3 progressive/generate.py',
        'version = 1',
        'routers = [' + ', '.join(f'{{ id = {r} }}' for r in routers) + ']',
        'lans = [',
    ]
    for i, (lan, router) in enumerate(zip(lans, routers)):
        lines.append(f'  {{ id = {lan}, router = {router} }}, # site {i // per_site + 1}; {sizes[i] + servers_per_lan + 1} total nodes')
    lines.extend([']', 'workstations = ['])
    for lan, members in zip(lans, hosts):
        for host in members:
            flag = ', compromised = true' if host in infected else ''
            lines.append(f'  {{ id = {host}, lan = {lan}{flag} }},')
    lines.extend([']', 'servers = ['])
    for lan, members in zip(lans, servers):
        lines.extend(f'  {{ id = {server}, lan = {lan} }},' for server in members)
    lines.extend([']', 'connections = ['])
    lines.extend(f'  {{ from = {a}, to = {b} }},' for a, b in sorted(edges))
    lines.extend([
        ']', '', '[simulation]', f'seed = {seed}', f'tick_limit = {deadline}', 'tick_ms = 0',
        '', '[mission]', 'enabled = true', 'recovery_ticks = 14', 'outage_ticks = 30',
        'service_threshold_percent = 90', 'services = [',
    ])
    lines.extend(f'  {{ client = {host}, server = {server}, weight = {weight} }},'
                 for host, server, weight in services)
    lines.append(']')
    path = Path(__file__).parent / f'{level:02}-{total}-nodes' / 'network.toml'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text('\n'.join(lines) + '\n', encoding='utf-8')
    print(f'{path}: {total:,} nodes / {count} LANs / {infected_count} initial infections')


if __name__ == '__main__':
    for level, settings in enumerate(LEVELS, start=1):
        generate(level, *settings)
