# tsconv

Converts timesheets between two plain-text formats:

- `csv` - what most payroll and invoicing tools export or import:
  ```
  date,employee,project,start,end
  2026-09-15,ashley,website-redesign,09:00,17:30
  ```
- `punch` - a flat text format some people just type by hand into a log file:
  ```
  2026-09-15 ashley website-redesign 09:00-17:30
  ```

I keep my own hours in `punch` because it's fast to type, but the invoicing
tool I bill through only takes `csv`. Every format in between (and there are
a few) wants one or the other, so this is a small converter plus a report
command for sanity-checking totals before they go anywhere.

Note: `punch` has no quoting, so employee and project names in that format
can't contain spaces (use `website-redesign`, not `website redesign`).

Shifts that cross midnight are fine - if the end time is earlier than the
start time, it's read as ending the next day (`22:00-06:00` is an 8 hour
shift). The entry's date stays the date the shift started on. A shift can't
be a full 24 hours or more, so a start and end time that are identical is
rejected as an error rather than guessed at.

## Build

```
cargo build --release
```

No third-party crates, so this also works offline with just `rustc` if you
don't want to pull in cargo's target machinery:

```
rustc src/main.rs -o tsconv
```

(that mode needs `timesheet.rs` and `report.rs` alongside `main.rs`, which
is already how the repo is laid out)

## Convert

```
$ cat week.csv
date,employee,project,start,end
2026-09-15,ashley,website-redesign,09:00,17:30
2026-09-16,ashley,website-redesign,09:00,16:00

$ tsconv convert --from csv --to punch --input week.csv
2026-09-15 ashley website-redesign 09:00-17:30
2026-09-16 ashley website-redesign 09:00-16:00
```

Write straight to a file instead of stdout with `--output`:

```
$ tsconv convert --from punch --to csv --input week.punch --output week.csv
```

By default a malformed row stops the whole conversion (strict mode), on the
theory that a typo in a timesheet is worth noticing rather than silently
dropping. Pass `--lenient` to skip bad rows instead and get a warning on
stderr for each one:

```
$ tsconv convert --from csv --to punch --input week.csv --lenient
warning: csv line 4: expected 5 fields (date,employee,project,start,end), found 4 (skipped)
2026-09-15 ashley website-redesign 09:00-17:30
2026-09-16 ashley website-redesign 09:00-16:00
```

## Report

Totals per employee and per project, either as a human-readable summary.
`--lenient` works here too, if the log has a few bad rows you'd rather skip
than fix before you can see the totals.

```
$ tsconv report --input week.csv --format csv
entries: 2
total:   16h 30m

by employee:
  ashley               16h 30m

by project:
  website-redesign     16h 30m
```

or as JSON, for feeding into something else:

```
$ tsconv report --input week.csv --format csv --json
{
  "entry_count": 2,
  "total_minutes": 990,
  "by_employee": {
    "ashley": 990
  },
  "by_project": {
    "website-redesign": 990
  }
}
```

## License

MIT, see [LICENSE](LICENSE).
