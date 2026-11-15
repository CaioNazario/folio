# Changelog

All notable changes to Folio. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions follow [SemVer](https://semver.org/).

## [Unreleased]

### Security

- The app window has a content security policy: documents cannot load anything from the network.

### Fixed

- Reloading the window no longer fails with "workspace already open in another window".
- Opening a document that is gone from disk removes it from the sidebar, recents and favorites instead of leaving it listed.

### Added

- Interface in Portuguese and English, following the system language.
- Window with the sidebar and the Folio logo.
- Sidebar with Favorites, Projects, Recent and footer, showing the open workspace: lazily loaded project tree, pins, recent documents and workspace totals.
- First run asks where to create the workspace, with the system Documents folder preselected; a missing workspace offers to retry or choose another.
- Text documents open in the reader and enter the recent list.
- Theme (light, dark, follow system) and language selectors in the sidebar footer.
- Theme and language choices are saved and restored on the next launch.
- Backend errors are shown in the active language.
- Document view with header, Convert and More menus, and a reader stage with zoom, fullscreen and keyboard shortcuts, showing TXT and Markdown documents.
- Markdown documents show their images, and clicked links open in the browser or mail app, open the linked document, or warn when Folio cannot open the file.
- The reader remembers how far each document was scrolled and returns there when it is opened again.
- Page Down, Page Up, the left and right arrows, Home and End scroll the reader one screen or to either end.
- Opening a document that is gone from disk refreshes the sidebar.
- Home screen, shown when no document is open and reached from the logo: quick actions, recent documents to continue reading, and project cards that open the project in the sidebar.
