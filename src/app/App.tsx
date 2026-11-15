// Shell da janela (Design/specs/02-layout.md): sidebar + área principal.
export function App() {
  return (
    <div className="flex h-screen w-full overflow-hidden bg-bg font-sans text-tx-2">
      <aside
        aria-label="Sidebar"
        className="flex min-w-[196px] flex-[0_0_min(272px,24vw)] flex-col border-r border-surf-4 bg-bg"
      />
      <main className="relative flex min-w-0 flex-1 flex-col" />
    </div>
  )
}
