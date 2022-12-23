# Papers

Papers is a paper repository management tool for ingesting, managing and searching papers.

## Create a repo

```sh
papers init
# creates sqlite db file
```

## Fetching

```sh
papers fetch --tag '<tag>' <url>...
# downloads pdfs at given urls to the current repo directory and adds the tag with them to the db
```

## Add

To add a file without fetching it run

```sh
papers add --tag '<tag>' <file>
```

## Listing

```sh
papers list
# list all

papers list --tags 'new'
# list all that have the tag 'new'
```

## Searching

```sh
papers search 'kv'
# search db for string
```