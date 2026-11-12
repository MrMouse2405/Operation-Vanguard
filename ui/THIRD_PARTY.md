# Third-party UI source

`src/lib/components/ui/`, supporting hooks/utilities, and the documentation
examples in `src/lib/examples/` originate from
[shadcn-svelte](https://github.com/huntabyte/shadcn-svelte), under the MIT license
in `SHADCN-LICENSE.md`. Component code was installed with shadcn-svelte CLI 1.8.0
(Nova, neutral). Example revision and the reproducible import-path adaptation
are recorded in `scripts/sync-examples.ts`.

Locally authored examples replace documentation examples that depend on the
upstream documentation app, and add composite examples. All content on the
showcase is demonstration data, not live simulator activity.

Local component adaptations preserve upstream APIs and apply shared corner-cut
classes, semantic theme tokens, and visible keyboard focus. The input component
also binds its public `ref` in the non-file branch.
