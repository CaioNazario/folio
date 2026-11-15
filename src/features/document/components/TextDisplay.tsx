// TXT view: the decoded text as is (specs/07-reader.md, "Exibição").
export function TextDisplay({ text }: { text: string }) {
  return (
    <div className="text-[calc(2.55cqw*var(--zoom))] leading-[1.62] break-words whitespace-pre-wrap">{text}</div>
  )
}
