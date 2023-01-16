# Papers

Papers is a paper repository management tool for ingesting, managing and searching papers.

It is inspired by the [papr](https://github.com/daniel-e/papr) tool used in [this blog post](https://segv.dev/paper-reading-workflow/).

## Create a repo

```sh
papers init
# creates sqlite db file
```

## Add

To add a file without fetching it run

```sh
papers add --tag '<tag>' <file|url>
```

## Listing

```sh
papers list
# list all

papers list --tags 'new'
# list all that have the tag 'new'
```

## Update some metadata about a paper

```sh
papers update <id>
```

## Notes

```sh
papers notes <id>
# edit notes for the paper id from list
```

## Open a paper file

```sh
papers open <id>
```