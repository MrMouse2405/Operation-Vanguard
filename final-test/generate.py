#!/usr/bin/env python3
"""Build the exact, explicit 10,000-node final fixture using only Python 3."""
from pathlib import Path
import random

SEED = 20261112
rng = random.Random(SEED)
routers = rng.sample(range(1, 997), 24)
lans = rng.sample(range(2000, 9000), 24)
# Four sites: one large campus, two regional LANs and three branches per site.
sizes = []
branch = 0
for site in range(4):
    sizes.extend([1200, 400, 400])
    for _ in range(3):
        sizes.append(161 if branch < 8 else 160)
        branch += 1
assert sum(sizes) == 9928
host_ids = list(range(10001, 10001 + 9928 * 7, 7))
rng.shuffle(host_ids)
servers = [[200003 + (2 * i + j) * 11 for j in range(2)] for i in range(24)]
hosts = []
cursor = 0
for size in sizes:
    hosts.append(sorted(host_ids[cursor:cursor + size]))
    cursor += size

# Neither ascending nor descending scans get all eight seeds quickly.
infected = {hosts[6 * site][-1] for site in range(4)}
infected.update([hosts[1][0], hosts[14][0], hosts[10][len(hosts[10]) // 2], hosts[23][len(hosts[23]) // 2]])
assert len(infected) == 8
edges = set()
def link(a, b):
    assert a != b
    edges.add(tuple(sorted((routers[a], routers[b]))))

for site in range(4):
    base = site * 6
    for offset in range(6):
        link(base + offset, base + (offset + 1) % 6)
    link(base + 1, base + 4)
    link(base, ((site + 1) % 4) * 6)
# Alternative inter-site links, without making a full mesh.
link(1, 13)
link(8, 20)

lines = [
    '# Operation Vanguard final: Four Sites, One Bad Morning',
    '# Exactly 10,000 nodes: 24 routers + 9,928 workstations + 48 servers.',
    '# 4 campus LANs (1,200 WS), 8 regional LANs (400 WS), 12 branches (160/161 WS).',
    '# 8 initial infections; 34 undirected router links; 29,784 required pairs.',
    '# Normal engine budgets/timings; no invisible waves, scripted events or cheats.',
    '# Each client requires local (weight 2), site (weight 2), remote (weight 3) service.',
    '# Generate with: python3 final-test/generate.py',
    'version = 1',
    'routers = [' + ', '.join(f'{{ id = {r} }}' for r in routers) + ']',
    'lans = [',
]
for i, (lan, router, size) in enumerate(zip(lans, routers, sizes)):
    role = ['campus', 'regional A', 'regional B', 'branch A', 'branch B', 'branch C'][i % 6]
    lines.append(f'  {{ id = {lan}, router = {router} }}, # site {i // 6 + 1}, {role}, {size} workstations + 2 servers')
lines.extend([']', 'workstations = ['])
for i, members in enumerate(hosts):
    lines.append(f'  # LAN {lans[i]}')
    for host in members:
        flag = ', compromised = true' if host in infected else ''
        lines.append(f'  {{ id = {host}, lan = {lans[i]}{flag} }},')
lines.extend([']', 'servers = ['])
for lan, pair in zip(lans, servers):
    lines.extend(f'  {{ id = {server}, lan = {lan} }},' for server in pair)
lines.extend([']', 'connections = ['])
lines.extend(f'  {{ from = {a}, to = {b} }},' for a, b in sorted(edges))
lines.extend([
    ']', '', '[simulation]', f'seed = {SEED}', 'tick_limit = 240', 'tick_ms = 0',
    '', '[mission]', 'enabled = true', 'recovery_ticks = 14', 'outage_ticks = 30',
    'service_threshold_percent = 90', 'services = [',
])
for i, members in enumerate(hosts):
    site, offset = divmod(i, 6)
    neighbour = site * 6 + (offset + 1) % 6
    remote = ((site + 1) % 4) * 6
    for index, host in enumerate(members):
        for server, weight in [(servers[i][index % 2], 2),
                               (servers[neighbour][index % 2], 2),
                               (servers[remote][index % 2], 3)]:
            lines.append(f'  {{ client = {host}, server = {server}, weight = {weight} }},')
lines.append(']')
path = Path(__file__).with_name('network.toml')
path.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(f'{path}: 10,000 nodes, {len(edges)} links, {len(infected)} initial infections, {sum(sizes) * 3:,} required pairs')
