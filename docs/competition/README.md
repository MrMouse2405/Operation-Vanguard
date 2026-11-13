# Operation Vanguard competition package

The package adapts the supplied RMC 2026–2027 Programming Competition brief to
Operation Vanguard. It retains four-person teams, seven development hours, a
two-page report, and a ten-minute presentation plus five minutes of questions.
The simulator contributes **50 / 100** competition points; engineering contributes
30, presentation 15, and resource management/attribution 5.

## Files

- `Operation-Vanguard-Competition-Package.pdf`: print/distribution copy.
- `Operation-Vanguard-Competition-Package.docx`: editable Word/LibreOffice copy.
- `Competition-Package.md`: canonical text, with explicit page boundaries.
- `build_package.py`: generates the DOCX from the Markdown source.

The source attachment's binary PDF and original logo assets were not available
in the workspace. This is a replacement document using the supplied content,
red/white identity and classification text; it does not reproduce unprovided logos
or credit a new author on behalf of RMC.

## Organizer release checklist

Before distribution, confirm the venue's submission deadline/channel and schedule,
the 100-point allocation and analytic rubric, and the published adjudication and
AI-use rules. Publish the official run manifest at the opening briefing: pinned
simulator revision, run count, scenario/seed policy, mission horizons/settings,
budgets, and the common debug setting. If held-out instances are used, announce
their family/count and freeze exact inputs before judging. The package defines
equal-weight averaging and documented reruns; it does not silently choose an
official suite or distribute an organizer solution.

The handbook's victory-first comparison advice is strategy guidance. This
competition package explicitly ranks by its combined 100-point assessment, with
the numerical mean run score supplying the 50-point simulator component; outcomes
are still recorded, without a second automatic outcome bonus.

## Rebuild

Maintainer tools: Python with `python-docx`, and LibreOffice for PDF conversion.

```sh
python3 build_package.py
libreoffice -env:UserInstallation=file:///tmp/vanguard-package-lo --headless --convert-to pdf --outdir . Operation-Vanguard-Competition-Package.docx
```

Review page breaks, rubric readability and the contents page after changing text.
The report's required Times New Roman/11 pt format is a competitor deliverable
rule; the package itself uses a sans-serif document layout.

## Rules checked against

- `src/scoring.rs`: service/security/recovery accounting, early termination,
  default verification/outage windows, and operational-host eligibility.
- `src/sim.rs`, `src/network.rs`, `src/main.rs`: host/route policies, five task slots,
  explicit topology, sandbox execution inputs and headless tick bound.
- `vandocs/content/mail/{briefing,operation,scoring,sandbox}.md`: learning path,
  OJE contacts, AI permissions, private VMs, and competition framing.

Only documentation artifacts are added; the competition weighting is a judging
conversion and does not change the simulator's 1,000-point scoring implementation.
