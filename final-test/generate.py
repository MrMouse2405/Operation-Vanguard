#!/usr/bin/env python3
"""Build the exact, explicit 10,000-node final fixture using only Python 3."""
from pathlib import Path
import random

SEED = 20261112
SITES = 4
LANS_PER_SITE = 10
ROUTER_COUNT = SITES * LANS_PER_SITE
WORKSTATION_COUNT = 10000 - 3 * ROUTER_COUNT
rng = random.Random(SEED)
routers = rng.sample(range(1, 997), ROUTER_COUNT)
lans = rng.sample(range(2000, 9000), ROUTER_COUNT)
# Each site has two campuses, three regional LANs and five branches.
# Include one router and two servers in the 500-node-per-LAN cap.
site_sizes = [497, 497, 348, 348, 248, 148, 98, 98, 94, 94]
sizes = site_sizes * SITES
assert sum(sizes) == WORKSTATION_COUNT
assert max(size + 3 for size in sizes) == 500
host_ids = list(range(10001, 10001 + WORKSTATION_COUNT * 7, 7))
rng.shuffle(host_ids)
servers = [[200003 + (2 * i + j) * 11 for j in range(2)] for i in range(ROUTER_COUNT)]
hosts = []
cursor = 0
for size in sizes:
    hosts.append(sorted(host_ids[cursor:cursor + size]))
    cursor += size

# Neither ascending nor descending scans get all eight seeds quickly.
infected = {hosts[LANS_PER_SITE * site][-1] for site in range(SITES)}
infected.update([hosts[2][0], hosts[23][0], hosts[16][len(hosts[16]) // 2], hosts[39][len(hosts[39]) // 2]])
assert len(infected) == 8
edges = set()
def link(a, b):
    assert a != b
    edges.add(tuple(sorted((routers[a], routers[b]))))

for site in range(SITES):
    base = site * LANS_PER_SITE
    for offset in range(LANS_PER_SITE):
        link(base + offset, base + (offset + 1) % LANS_PER_SITE)
    link(base + 1, base + 5)
    link(base + 2, base + 7)
    link(base, ((site + 1) % SITES) * LANS_PER_SITE)
# Alternative inter-site links, without making a full mesh.
link(1, 21)
link(12, 32)

lines = [
    '# Operation Vanguard final: Four Sites, One Bad Morning',
    '# Exactly 10,000 nodes: 40 routers + 9,880 workstations + 80 servers.',
    '# Four sites, each with 10 LANs. Maximum 500 nodes/LAN INCLUDING router + servers.',
    '# Workstations/LAN: 497, 497, 348, 348, 248, 148, 98, 98, 94, 94 per site.',
    '# 8 initial infections; 54 undirected router links; 29,640 required pairs.',
    '# Normal engine budgets/timings; no invisible waves, scripted events or cheats.',
    '# Each client requires local (weight 2), site (weight 2), remote (weight 3) service.',
    '# Generate with: python3 final-test/generate.py',
    'version = 1',
    'routers = [' + ', '.join(f'{{ id = {r} }}' for r in routers) + ']',
    'lans = [',
]
for i, (lan, router, size) in enumerate(zip(lans, routers, sizes)):
    role = ['campus A', 'campus B', 'regional A', 'regional B', 'regional C',
            'branch A', 'branch B', 'branch C', 'branch D', 'branch E'][i % LANS_PER_SITE]
    lines.append(f'  {{ id = {lan}, router = {router} }}, # site {i // LANS_PER_SITE + 1}, {role}, {size + 3} nodes: {size} workstations + 2 servers + router')
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
    site, offset = divmod(i, LANS_PER_SITE)
    neighbour = site * LANS_PER_SITE + (offset + 1) % LANS_PER_SITE
    remote = ((site + 1) % SITES) * LANS_PER_SITE
    for index, host in enumerate(members):
        for server, weight in [(servers[i][index % 2], 2),
                               (servers[neighbour][index % 2], 2),
                               (servers[remote][index % 2], 3)]:
            lines.append(f'  {{ client = {host}, server = {server}, weight = {weight} }},')
lines.append(']')
path = Path(__file__).with_name('network.toml')
path.write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(f'{path}: 10,000 nodes, {len(edges)} links, {len(infected)} initial infections, {sum(sizes) * 3:,} required pairs')
