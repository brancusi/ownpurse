# Backing up and restoring a record

A record is the XTDB node's files in the data directory (`OWNPURSE_DATA_DIR`, default `data`): `xtdb/log` and
`xtdb/objects`, and the files the record keeps beside them in `files/sha256` (such as attachments' files), each
named for the SHA-256 of its bytes. Copying that folder while ownpurse runs can catch a file half-written, so take a snapshot instead.

## Taking a snapshot

While ownpurse runs:

```sh
curl -s -X POST localhost:7070/backup -H 'content-type: application/json' -d '{"dir": "/Volumes/backup/ownpurse"}'
```

or `(backup "/Volumes/backup/ownpurse")` at the REPL. `dir` is an absolute path; ownpurse creates it when it is
missing. Each snapshot is a new folder in it, `ownpurse-<time>`, holding `xtdb/objects`, `files/sha256` and
`snapshot.edn`.

What happens:

1. ownpurse asks XTDB to finish its current block, which writes every transaction so far to storage files. XTDB
   writes each storage file once and never changes it.
2. It copies every storage file up to that block into `.ownpurse-<time>.partial` in `dir`.
3. It copies the record's kept files. Each is written once, to a scratch name first, and never changed, and the
   record names a file only after it is in place, so every file the copied rows name is there.
4. It opens the copy on a separate node and reads every version of every table. It also checks that the copy holds
   as many transactions as the record did. If anything is wrong, it deletes the partial folder and answers with
   an error.
5. It renames the folder to `ownpurse-<time>`, so a folder with that name is always a complete snapshot. It
   then notes the snapshot in the record's `backups` table.

A transaction written while the snapshot runs is not in it; it is in the next one. Each snapshot holds the whole
record (there are no incremental snapshots), so delete old ones as you see fit.

`GET /health` returns `last-snapshot` (`dir`, `taken-at`, `block`, `transactions`) once the record has one. A
workspace can warn when `taken-at` is too old. A restored record's last snapshot is the one before the restore:
the snapshot is noted in the record after it is taken, so the snapshot itself does not hold that note.

## Restoring

With ownpurse stopped, into a data directory that holds no record:

```sh
clojure -M:dev -m ownpurse.api.main restore /Volumes/backup/ownpurse/ownpurse-20261026T154641Z /path/to/new-data
OWNPURSE_DATA_DIR=/path/to/new-data clojure -M:dev -m ownpurse.api.main
```

The data directory defaults to `OWNPURSE_DATA_DIR`. A restore never writes over a record: move the old one aside
first.

What the restored record holds:

- Every version of every row, with its valid time **and its system time** as they were. `FOR ALL SYSTEM_TIME`
  queries give the same answers as on the original, up to the snapshot.
- `xt.txs` as it was: every transaction's id, system time, whether it committed, and its metadata (`source`,
  `why`).
- The kept files, in `files/sha256`.
- A new log. XTDB requires a node whose storage is ahead of its log to start a new log generation (an **epoch**),
  so the restore writes the next epoch to `xtdb/epoch.edn`, and ownpurse starts the node with it. Transactions after
  the restore get ids in the new epoch, which sort after every earlier id. Keep `epoch.edn` with the record.

## A full-history export

An export of every table `FOR ALL VALID_TIME FOR ALL SYSTEM_TIME` (over the read-only SQL port) is readable on its
own, but ownpurse cannot restore from it yet. XTDB sets system time itself when it writes, so a replay of the
export would bring back every valid time, while the system times and `xt.txs` would be those of the replay. The
originals could only be kept as data beside it. Use snapshots to restore; keep exports as a readable copy.
