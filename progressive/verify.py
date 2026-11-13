#!/usr/bin/env python3
"""Run the reference defender through complete missions and retain compact evidence.

Python standard library only. Uses the normal simulator executable and replay
format; a partial run or a Lua error is a verification failure, not a victory.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import shutil
import tempfile
import time
import tomllib


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    root = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--vanguard", default="vanguard", help="installed command or simulator binary path")
    parser.add_argument("--agent", type=Path, default=root / "containment-defender.lua")
    parser.add_argument("--output", type=Path, default=root / "validation-results.json")
    parser.add_argument("--levels", nargs="+", type=int, help="optional level numbers, e.g. --levels 1 6 10 11")
    parser.add_argument("--replay-dir", type=Path, help="retain raw replay files here (large for level 11)")
    args = parser.parse_args()
    agent = args.agent.resolve()
    if not agent.is_file():
        parser.error(f"agent not found: {agent}")
    networks = sorted(root.glob("[0-9][0-9]-*/network.toml"))
    if args.levels:
        available = {int(p.parent.name.split("-")[0]) for p in networks}
        if set(args.levels) - available:
            parser.error("requested level does not exist")
        networks = [p for p in networks if int(p.parent.name.split("-")[0]) in args.levels]
    version = subprocess.run([args.vanguard, "--version"], capture_output=True, text=True, check=True).stdout.strip()
    binary = shutil.which(args.vanguard)
    results = {"agent": agent.name, "agent_sha256": digest(agent), "simulator": version,
               "simulator_binary_sha256": digest(Path(binary)) if binary else None,
               "expected_runs": len(networks), "passed": False, "runs": []}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="vanguard-progressive-") as temporary:
        replay_dir = args.replay_dir or Path(temporary)
        replay_dir.mkdir(parents=True, exist_ok=True)
        for network in networks:
            config = tomllib.loads(network.read_text())
            horizon = config["simulation"]["tick_limit"]
            replay = replay_dir / f"{network.parent.name}.json"
            agent_path = agent.relative_to(Path.cwd()) if agent.is_relative_to(Path.cwd()) else agent
            network_path = network.relative_to(Path.cwd()) if network.is_relative_to(Path.cwd()) else network
            command = [args.vanguard, "--script", str(agent_path), "--network", str(network_path),
                       "--headless", "--ticks", str(horizon), "--replay", str(replay)]
            started = time.perf_counter()
            process = subprocess.run(command, capture_output=True, text=True)
            run = {"network": f"{network.parent.name}/network.toml", "network_sha256": digest(network),
                   "seed": config["simulation"]["seed"], "deadline": horizon,
                   "command": command, "wall_seconds": round(time.perf_counter() - started, 3)}
            if process.returncode:
                run.update(passed=False, returncode=process.returncode, error=process.stderr.strip())
            else:
                with replay.open() as handle:
                    data = json.load(handle)
                snapshot = data["snapshot"]
                score = snapshot["score"]
                nodes = snapshot["nodes"]
                errors = sum(agent["runtime_errors"] for agent in snapshot["agents"])
                compromised = sum(bool(node["compromised"]) for node in nodes)
                exposed = sum(node.get("exposure") is not None for node in nodes)
                run.update(outcome=score["outcome"], tick=snapshot["tick"], score=score["total"],
                           service=score["service"], security=score["security"], recovery=score["recovery"],
                           final_service_fraction=score["service_fraction"], runtime_errors=errors,
                           compromised=compromised, exposed=exposed,
                           semantic_hash=data["hashes"][-1]["hash"], hashes=len(data["hashes"]),
                           total_nodes=len(nodes), routers=len(snapshot["agents"]))
                run["passed"] = (snapshot["control"] == "FINISHED" and score["outcome"] == "VICTORY"
                                 and not errors and not compromised and not exposed)
                del data, snapshot, nodes
            results["runs"].append(run)
            results["passed"] = (len(results["runs"]) == len(networks)
                                 and all(item["passed"] for item in results["runs"]))
            args.output.write_text(json.dumps(results, indent=2) + "\n")
            print(f"{network.parent.name}: {run.get('outcome', 'ERROR')} T{run.get('tick', '?')} "
                  f"score={run.get('score', '?')} errors={run.get('runtime_errors', '?')}", flush=True)
            if not args.replay_dir and replay.exists():
                replay.unlink()
    if not results["passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
