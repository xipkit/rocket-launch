# Launch day runbook

> [!warning] Read before T-30
> Every step below is a *console* step. Pad steps are in the pad crew's own
> book and are not repeated here.

## T-30:00 · Poll

1. Run `ground-control status`. All rows must read **GO**.
2. If `range` is NO-GO, stop. Do not enter the terminal count.
3. Confirm the weather balloon report is under 20 minutes old.

## T-10:00 · Terminal count

```sh
cargo run --release --bin sequencer -- --mission mission.toml
```

The sequencer prints one line per second. Watch for `HOLD`.

> [!danger] Auto-abort window
> Inside T-10 the sequencer will **abort** instead of holding on a redline.
> There is no operator action that stops this, by design.

## Holds

| Hold point | Who releases | Typical cause                |
| ---------- | ------------ | ---------------------------- |
| T-30:00    | flight       | range or weather             |
| T-10:00    | flight       | propellant temperatures      |
| T-01:00    | pad          | ullage pressure settling     |
| T-00:10    | nobody       | redline here is an abort     |

> [!tip]
> A hold at T-01:00 for LOX ullage usually clears itself in under two
> minutes. Do not recycle the count for it.

## Aborts

The sequencer prints the abort mode and the safing sequence for it. Read the
safing line on the loop, then hand over to the pad crew.

| Mode              | Safing                              |
| ----------------- | ----------------------------------- |
| Chamber pressure  | close main valves, purge chamber    |
| Ullage            | vent LOX to 2.0 bar, hold RP-1      |
| Bus               | swap to ground power, disarm FTS    |

## After liftoff

- [ ] Confirm MECO on the dashboard
- [ ] Save the downlink capture: `scripts/countdown.sh --archive`
- [ ] File the flight report using the template in [[Flight Report]]

%% Operators: the archive step is the one people forget. %%
