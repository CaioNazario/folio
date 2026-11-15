// Page typography (Design/specs/05-document-view.md, 5.5), applied to the
// elements of the rendered document. Sizes are cqw of the page times --zoom.
export const PAGE_TYPOGRAPHY = [
  '[&_p]:mt-0 [&_p]:mb-[calc(1.8cqw*var(--zoom))] [&_p]:text-[calc(2.55cqw*var(--zoom))] [&_p]:leading-[1.62]',
  '[&_h1]:mt-[calc(2cqw*var(--zoom))] [&_h1]:mb-[calc(0.8cqw*var(--zoom))] [&_h1]:text-[calc(3.9cqw*var(--zoom))] [&_h1]:leading-[1.25] [&_h1]:font-semibold [&_h1]:text-pretty',
  '[&_:is(h2,h3,h4,h5,h6)]:mt-[calc(2.6cqw*var(--zoom))] [&_:is(h2,h3,h4,h5,h6)]:mb-[calc(1.1cqw*var(--zoom))] [&_:is(h2,h3,h4,h5,h6)]:text-[calc(2.75cqw*var(--zoom))] [&_:is(h2,h3,h4,h5,h6)]:leading-[1.4] [&_:is(h2,h3,h4,h5,h6)]:font-semibold',
  '[&_:is(ul,ol)]:mt-0 [&_:is(ul,ol)]:mb-[calc(1.8cqw*var(--zoom))] [&_:is(ul,ol)]:pl-[1.4em] [&_ul]:list-disc [&_ol]:list-decimal [&_li]:text-[calc(2.55cqw*var(--zoom))] [&_li]:leading-[1.62]',
  '[&_blockquote]:mx-0 [&_blockquote]:border-l-2 [&_blockquote]:border-paper-rule [&_blockquote]:pl-[1em] [&_blockquote]:text-paper-mut2 [&_blockquote]:italic',
  '[&_hr]:my-[calc(2.3cqw*var(--zoom))] [&_hr]:h-px [&_hr]:border-none [&_hr]:bg-paper-rule',
  '[&_code]:font-mono [&_code]:text-[0.9em] [&_pre]:overflow-x-auto [&_pre]:rounded-7 [&_pre]:bg-paper-alt [&_pre]:p-[1em] [&_pre]:text-[calc(2.2cqw*var(--zoom))]',
  '[&_table]:mb-[calc(1.8cqw*var(--zoom))] [&_table]:border-collapse [&_:is(td,th)]:border [&_:is(td,th)]:border-paper-rule [&_:is(td,th)]:px-[0.6em] [&_:is(td,th)]:py-[0.3em] [&_:is(td,th)]:text-[calc(2.3cqw*var(--zoom))]',
  '[&_a]:text-paper-tx [&_a]:underline [&_img]:max-w-full',
].join(' ')
