# Changelog

All notable changes to Folio. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions follow [SemVer](https://semver.org/).

## [Unreleased]

### Added

- Converting a PDF keeps its images: they are extracted in reading order, and a scanned PDF becomes a document of images (Closes #1).
- Converting a PDF to DOCX keeps the font and size of the body and the size, font and weight of each heading level.

### Changed

- Folio is now licensed under the GPL-3.0-or-later (previously MIT).

### Security

- The app window has a content security policy: documents cannot load anything from the network.
- Links clicked in a document are opened by the backend; the window itself can no longer open URLs.

### Fixed

- Reloading the window no longer fails with "workspace already open in another window".
- Opening a document that is gone from disk removes it from the sidebar, recents and favorites instead of leaving it listed.
- The scrollbar hover and the avatar letter follow the light theme.

### Added

- Convert the open document to PDF, Word, Markdown or plain text from the Converter menu. The new file is created next to the original, which is never changed, and opens when ready. Conversions run in the background one at a time, show their progress and can be cancelled; converting a password-protected document warns that the copy will have no password.
- Converting a Markdown document keeps its local images in the Word and PDF copies; web images are not downloaded and become links.
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
- PDF documents open in the reader, with continuous pages, the current page and total in the toolbar, zoom, page keys and a password prompt for protected files; they reopen on the page where the reader stopped.
- Word documents (DOCX) open in the reader, with pages, the current page and total in the toolbar, zoom, page keys and a password prompt for protected files; images anchored to the page and bitmaps saved as EMF show in place, and the document reopens on the page where the reader stopped.
- Page Down, Page Up, the left and right arrows, Home and End scroll the reader one screen or to either end.
- Opening a document that is gone from disk refreshes the sidebar.
- Home screen, shown when no document is open and reached from the logo: quick actions, recent documents to continue reading, and project cards that open the project in the sidebar.
- Links to a heading inside a Markdown document scroll to that heading.
- Projects and Recent in the sidebar say when they are empty.
- Properties dialog, from the More menu: file facts, the document's own metadata (title, author, pages and more) and its conversions, with warnings when the original or the converted file changed; a converted document opens from there.
