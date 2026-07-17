# Design note: Auto compress jobs (`--at`)

## Path

Fast Path feature on pack path; pure formula in domain + CLI/config wiring.

## Decision summary

See **ADR 0025**. One-shot N after plan; I/O-aware cap; no mid-run pool resize.

## Why not dynamic up/down (for now)

- Idle workers yield CPU to other processes.
- Transfer is already serial; over-compress mostly hurts **disk contention / stage**.
- Plan-time `io_cap` from `--connection` addresses the main failure mode.
- Elastic rayon is high cost / low certainty; revisit with queue backpressure if needed.

## Operator recipe

```bash
ax pa -i SRC -o DEST -s 4G --connection usb3 --at --work-dir /tmp/fast-stage -f zip
```

- Fast local stage + slow USB: auto keeps compress modest (usb3 class).
- Force: `-j 4` (ignores `--at`).
