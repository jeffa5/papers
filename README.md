# Papers

Papers is a paper repository management tool for ingesting, managing and searching papers.

## Create a repo

```sh
papers init
# creates sqlite db file
```

## Fetching

```sh
papers fetch --tag '<tag>' <url>
# downloads pdf at the given url to the current repo directory and adds the tags with them to the db
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

## Notes

```sh
papers notes <id>
# edit notes for the paper id from list
```