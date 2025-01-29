How `dandidav` Uses the DANDI Archive API
=========================================

*This document is up-to-date as of 2025 January 16.*

When `dandidav` receives a request for a path under `/dandisets/`, the requests
it makes to the DANDI Archive API for each type of `dandidav` request path are
as follows.

Note that there are two different approaches that `dandidav` uses for fetching
resources:

- When responding to a `GET` request or a `PROPFIND` request with a `Depth`
  header of `1` (hereafter referred to as a "deep request"), `dandidav`
  requests both the resource identified by the request path and (if that
  resource is a collection) all of its immediate child resources.

- When responding to a `PROPFIND` request with a `Depth` header of `0`
  (hereafter referred to as a "shallow request"), `dandidav` requests only the
  resource identified by the request path, not any of its children.

Dandiset Index
--------------

> **dandidav path:** `/dandisets/`

For deep requests, `dandidav` paginates over the API's `/dandisets/` endpoint.

For shallow requests, `dandidav` does not make any API requests.

Dandiset Top Level
------------------

> **dandidav path:** `/dandisets/{dandiset_id}/`

For both deep and shallow requests, `dandidav` makes an API request to `/dandisets/{dandiset_id}/`.

Releases Index
--------------

> **dandidav path:** `/dandisets/{dandiset_id}/releases/`

For deep requests, `dandidav` paginates over the API's `/dandisets/{dandiset_id}/versions/` endpoint.

For shallow requests, `dandidav` does not make any API requests.

Top Level of a Dandiset Version
-------------------------------

> **dandidav paths:**
>
> - `/dandiset/{dandiset_id}/draft/`
> - `/dandiset/{dandiset_id}/latest/`
> - `/dandiset/{dandiset_id}/releases/{version_id}/`

For both deep and shallow requests, if the request path is
`/dandiset/{dandiset_id}/latest/`, an initial request is made to
`/dandiset/{dandiset_id}/` to get the version ID of the latest published
version.

For deep requests, `dandidav` makes API requests to the following endpoints:

- `/dandisets/{dandiset_id}/versions/{version_id}/info/`
- `/dandisets/{dandiset_id}/versions/{version_id}/assets/paths/` (paginated)
    - Because this endpoint only provides assets' paths, IDs, and sizes, for
      each separate asset returned from this endpoint, `dandidav` makes another
      request to
      `/dandisets/{dandiset_id}/versions/{version_id}/assets/{asset_id}/info/`
      to fetch further details about the asset (See ["Other
      Notes"](#other-notes) below).
- Prior to [PR #236](https://github.com/dandi/dandidav/pull/236), a request was
  also made to `/dandisets/{dandiset_id}/versions/{version_id}/` to fetch the
  version metadata (so that its size could be reported), but this was changed
  to use the metadata from the
  `/dandisets/{dandiset_id}/versions/{version_id}/info/` request instead.

For shallow requests, `dandidav` makes an API request to
`/dandisets/{dandiset_id}/versions/{version_id}/info/`.

Metadata File
-------------

> **dandidav paths:**
>
> - `/dandiset/{dandiset_id}/draft/dandiset.yaml`
> - `/dandiset/{dandiset_id}/latest/dandiset.yaml`
> - `/dandiset/{dandiset_id}/releases/{version_id}/dandiset.yaml`

For both deep and shallow requests, if the request path is
`/dandiset/{dandiset_id}/latest/`, an initial request is made to
`/dandiset/{dandiset_id}/` to get the version ID of the latest published
version.  Then, for both deep and shallow requests, `dandidav` makes an API
request to `/dandisets/{dandiset_id}/versions/{version_id}/`.

Asset Path
----------

> **dandidav paths:**
>
> - `/dandiset/{dandiset_id}/draft/{path}`
> - `/dandiset/{dandiset_id}/latest/{path}`
> - `/dandiset/{dandiset_id}/releases/{version_id}/{path}`
>
> Note that any trailing slashes at the end of `path` are ignored and are
> stripped before passing to the Archive.

For both deep and shallow requests, if the request path is
`/dandiset/{dandiset_id}/latest/`, an initial request is made to
`/dandiset/{dandiset_id}/` to get the version ID of the latest published
version.

Then, for each initial subpath of `path` that ends with either (a) a component
ending in ".zarr" or ".ngff" (case insensitive) or (b) the end of the path, an
API request is made to
`/dandisets/{dandiset_id}/versions/{version_id}/assets/?path={subpath}&metadata=1&order=path`,
which is paginated through until one of the following occurs:

- An asset whose path equals `subpath` is found.  In this case:

    - If the asset is a blob:

        - If `subpath` is the whole of `path`, `dandidav` has found a blob
          asset, and the checking of initial subpaths terminates.

        - If `subpath` is not the whole of `path`, then the user requested a
          path under a blob, and so no more requests are made to the Archive,
          and `dandidav` returns a 404 response.

    - If the asset is a Zarr, `dandidav` has found a Zarr asset, and the
      checking of initial subpaths terminates.  If `subpath` is not the whole
      of `path`, a request is made to S3 to fetch information about the
      resource at the path equal to the remainder.

- An asset whose path is under the directory `{subpath}/` is found.  In this
  case, the next initial subpath is checked.  If there is no next initial
  subpath, then `dandidav` has found an asset folder at `path`.

- An asset whose path is lexicographically after `{subpath}/` is found, or no
  assets were returned by the API.  In this case, no more requests are made to
  the Archive, and `dandidav` returns a 404 response.

Shallow requests stop making requests to the Archive at this point.  Deep
requests continue as follows:

- If an asset folder was found at path `path`, a paginated request is made to
  `/dandisets/{dandiset_id}/versions/{version_id}/assets/paths/?path_prefix={path}/`.
  Because this endpoint only provides assets' paths, IDs, and sizes, for each
  separate asset returned from this endpoint, `dandidav` makes another request
  to `/dandisets/{dandiset_id}/versions/{version_id}/assets/{asset_id}/info/`
  to fetch further details about the asset (See ["Other Notes"](#other-notes)
  below).

- If a blob asset was found, no more requests are made to the Archive.

- If a Zarr asset was found, further requests are made to S3 to fetch
  information about the Zarr's entries.

Other Notes
-----------

- Paginated requests to the Archive use the default `page_size` value and are
  requested serially.

- `dandidav` requires the following information from the Archive about each
  type of resource, not always at the same time.  Required information about
  the relationships between resources is not listed here.

    - Dandisets:
        - identifier (to serve as the resource name)
        - creation & modification timestamps

    - Versions:
        - identifier (to serve as the resource name)
        - size
        - creation & modification timestamps
        - metadata (to serve the virtual `dandiset.yaml` resources)

    - Asset folders:
        - path/name

    - Assets:
        - path/name
        - ID (for computing the metadata URL to display in the web view)
        - blob ID and Zarr ID (to determine the asset type)
        - size
        - creation & modification timestamps
        - metadata:

            - `digest` — For blob assets, the `"dandi:dandi-etag"` digest is
              used as the resource's ETag in `PROPFIND` responses

            - `contentUrl` — As a reminder, this array contains both an Archive
              download URL and a direct S3 URL.  Which one is used depends on
              the asset type and the situation:

                - Blob assets: When rendering links in the web (HTML) view, the
                  Archive download URL is used, as this results in a redirect
                  to a signed S3 URL that sets the Content-Disposition header,
                  thereby ensuring that the user's browser downloads the asset
                  to a file of the same name.  When responding to `GET`
                  requests for blob assets, either URL may be used, depending
                  on the presence of the `--prefer-s3-redirects` command-line
                  option.

                - Zarr assets: The S3 URL is parsed by `dandidav` to locate the
                  Zarr's entries on S3.

            - `encodingFormat` — Used as the resource's Content Type in
              `PROPFIND` responses
