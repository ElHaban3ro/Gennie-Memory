import { useEffect, useMemo, useState } from 'react'

type Search = { query: string; scope_kind: string; scope_id: string; entity_id: string | null; as_of: string; limit: number; query_mode: 'keywords' | 'natural' }
type Candidate = { memory_id: string; revision: number; summary: string; conditions: string | null; source: { kind: string; ref: string; locator: string | null } | null; state: string; bm25: number }
type Verdict = 'match' | 'no_match' | 'unsure'
type Outcome = 'matches' | 'none' | 'missing'
type Decision = { memory_id: string; revision: number; verdict: Verdict }
type Review = { id: number; case_id: string | null; decisions: Decision[]; outcome: Outcome; note: string | null; created_at: string }
type View = { candidates: Candidate[]; review: Review | null; reusable: boolean }
type Case = { id: string; segment: string; search: Search }
type ExportRow = { id: number; case_id: string | null; query: string; scope_id: string; candidates: Candidate[]; decisions: Decision[]; outcome: Outcome; created_at: string }

const blank = (): Search => ({ query: '', scope_kind: 'conversation', scope_id: '', entity_id: null, as_of: new Date().toISOString(), limit: 20, query_mode: 'keywords' })

async function api<T>(path: string, body?: unknown): Promise<T> {
  const headers: Record<string, string> = {}
  if (body !== undefined) headers['Content-Type'] = 'application/json'
  const response = await fetch(path, { method: body === undefined ? 'GET' : 'POST', headers, body: body === undefined ? undefined : JSON.stringify(body) })
  const data = await response.json()
  if (!response.ok) throw new Error(data.error || `Error HTTP ${response.status}`)
  return data as T
}

function dateLabel(value: string) { try { return new Date(value).toLocaleString('es', { dateStyle: 'medium', timeStyle: 'short' }) } catch { return value } }

export default function App() {
  const [cases, setCases] = useState<Case[]>([])
  const [reviewedIds, setReviewedIds] = useState<string[]>([])
  const [filter, setFilter] = useState('conversation/response_style')
  const [caseId, setCaseId] = useState<string | null>(null)
  const [search, setSearch] = useState<Search>(blank)
  const [mode, setMode] = useState<'search' | 'scope'>('search')
  const [view, setView] = useState<View | null>(null)
  const [visibleCount, setVisibleCount] = useState(5)
  const [decisions, setDecisions] = useState<Record<string, Verdict>>({})
  const [outcome, setOutcome] = useState<Outcome | null>(null)
  const [note, setNote] = useState('')
  const [dirty, setDirty] = useState(false)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [message, setMessage] = useState('')
  const [exportRows, setExportRows] = useState<ExportRow[] | null>(null)
  const [chosenExports, setChosenExports] = useState<number[]>([])

  useEffect(() => { api<{ cases: Case[]; reviewed_ids: string[] }>('/api/cases').then(data => { setCases(data.cases); setReviewedIds(data.reviewed_ids) }).catch(e => setError(String(e))) }, [])
  useEffect(() => { const guard = (e: BeforeUnloadEvent) => { if (dirty) e.preventDefault() }; window.addEventListener('beforeunload', guard); return () => window.removeEventListener('beforeunload', guard) }, [dirty])

  const visibleCases = useMemo(() => cases.filter(item => filter === 'all' || item.segment === filter), [cases, filter])
  const segments = useMemo(() => [...new Set(cases.map(item => item.segment))].sort(), [cases])
  const matches = Object.values(decisions).filter(value => value === 'match').length
  const currentReviewed = visibleCases.filter(item => reviewedIds.includes(item.id)).length
  const shownCandidates = view?.candidates.slice(0, visibleCount) || []
  const partialSearch = mode === 'search' && !!view && visibleCount < view.candidates.length

  async function load(next: Search, nextMode: 'search' | 'scope', nextCase: string | null, jumpToEvidence = false) {
    setBusy(true); setError(''); setMessage('')
    try {
      const result = await api<View>('/api/view', { case_id: nextCase, search: next, view_mode: nextMode })
      setSearch(next); setMode(nextMode); setCaseId(nextCase); setView(result)
      const lastDecidedIndex = result.candidates.reduce((last, candidate, index) => result.review?.decisions.some(decision => decision.memory_id === candidate.memory_id) ? index : last, -1)
      setVisibleCount(nextMode === 'scope' ? result.candidates.length : Math.max(5, lastDecidedIndex + 1))
      setDecisions(Object.fromEntries((result.review?.decisions || []).map(item => [item.memory_id, item.verdict])))
      setOutcome(result.review?.outcome || null); setNote(result.review?.note || ''); setDirty(false)
      if (jumpToEvidence && window.matchMedia('(max-width: 800px)').matches) requestAnimationFrame(() => document.getElementById('evidence')?.scrollIntoView())
    } catch (e) { setError((e as Error).message) } finally { setBusy(false) }
  }
  function mayLeave() { return !dirty || window.confirm('Hay cambios sin guardar. ¿Quieres descartarlos?') }
  function chooseCase(item: Case) { if (mayLeave()) void load({ ...item.search, query_mode: item.search.query_mode || 'keywords', limit: item.search.limit || 20 }, 'search', item.id, true) }
  function newReview() { if (!mayLeave()) return; setCaseId(null); setSearch(blank()); setMode('search'); setView(null); setVisibleCount(5); setDecisions({}); setOutcome(null); setNote(''); setDirty(false); setError(''); setMessage('') }
  function changeMode(nextMode: 'search' | 'scope') {
    if (nextMode === mode || !mayLeave()) return
    if (view) void load(search, nextMode, caseId)
    else setMode(nextMode)
  }
  function setField<K extends keyof Search>(field: K, value: Search[K]) { setSearch(previous => ({ ...previous, [field]: value })); setDirty(true) }
  function mark(candidate: Candidate, verdict: Verdict) {
    const next = { ...decisions, [candidate.memory_id]: verdict }
    setDecisions(next)
    if (verdict === 'match') setOutcome('matches')
    else if (outcome === 'matches' && !Object.values(next).includes('match')) setOutcome(null)
    setDirty(true)
  }
  function selectOutcome(value: Outcome) {
    if (value === 'none' && partialSearch) { setError('Solo has visto parte de los resultados. Muestra los restantes antes de afirmar que ninguno responde.'); return }
    if (value !== 'matches') setDecisions(previous => Object.fromEntries(Object.entries(previous).filter(([, verdict]) => verdict !== 'match')))
    setOutcome(value); setDirty(true)
  }
  async function save() {
    if (!view || !outcome) return
    if (outcome === 'none' && partialSearch) { setError('Muestra todos los resultados antes de guardar «Ninguno responde».'); return }
    setBusy(true); setError(''); setMessage('')
    try {
      const items = view.candidates.map(candidate => ({ memory_id: candidate.memory_id, revision: candidate.revision }))
      const chosen: Decision[] = view.candidates.filter(candidate => decisions[candidate.memory_id]).map(candidate => ({ memory_id: candidate.memory_id, revision: candidate.revision, verdict: decisions[candidate.memory_id] }))
      const saved = await api<Review>('/api/reviews', { case_id: caseId, search, view_mode: mode, candidates: items, decisions: chosen, outcome, note: note.trim() || null })
      setView({ ...view, review: saved }); setDirty(false); setMessage(`Revisión #${saved.id} guardada en esta Mac.`)
      if (caseId && !reviewedIds.includes(caseId)) setReviewedIds(ids => [...ids, caseId])
    } catch (e) { setError((e as Error).message) } finally { setBusy(false) }
  }
  async function openExport() {
    setBusy(true); setError('')
    try { const result = await api<{ reviews: ExportRow[] }>('/api/export'); setExportRows(result.reviews); setChosenExports([]) }
    catch (e) { setError((e as Error).message) } finally { setBusy(false) }
  }
  function download() {
    const selected = (exportRows || []).filter(row => chosenExports.includes(row.id))
    if (!selected.length) return
    const blob = new Blob([JSON.stringify({ schema: 'iromem-dashboard-review-v1', reviews: selected }, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob); const link = document.createElement('a'); link.href = url; link.download = 'iromem-revisiones-seleccionadas.json'; link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000)
    setExportRows(null); setMessage(`${selected.length} revisiones exportadas. El archivo permanece bajo tu control.`)
  }

  return <div className="app-shell">
    <header className="topbar"><div className="brand"><span className="brand-mark">i<span>R</span></span><span>iROMEM <small>/ Dashboard</small></span></div><div className="top-actions"><span className="local-pill"><span className="live-dot" /> Local · esta Mac</span><button className="ghost" onClick={() => void openExport()}>Compartir / exportar <span aria-hidden="true">↗</span></button></div></header>
    <div className="workspace">
      <aside className="sidebar" aria-label="Preguntas para revisar">
        <div className="sidebar-heading"><div><p className="eyebrow">Espacio de trabajo</p><h2>Revisión de memoria</h2></div><button className="square-button" onClick={newReview} title="Nueva pregunta" aria-label="Nueva pregunta">＋</button></div>
        <div className="progress"><div className="progress-top"><span>Preguntas con feedback</span><strong>{currentReviewed} / {visibleCases.length}</strong></div><div className="progress-track"><span style={{ width: `${visibleCases.length ? currentReviewed / visibleCases.length * 100 : 0}%` }} /></div></div>
        <label className="filter-label" htmlFor="segment">CONJUNTO</label><select id="segment" value={filter} onChange={e => setFilter(e.target.value)}><option value="all">Todos los casos</option>{segments.map(segment => <option key={segment} value={segment}>{segment.replace('conversation/', 'Conversación · ')}</option>)}</select>
        <div className="queue-label"><span>PREGUNTAS</span><span>{visibleCases.length}</span></div>
        <div className="case-list">{visibleCases.map((item, index) => <button className={`case-item ${caseId === item.id ? 'active' : ''}`} key={item.id} onClick={() => chooseCase(item)}><span className="case-index">{String(index + 1).padStart(2, '0')}</span><span className="case-copy"><strong>{item.search.query}</strong><small>{item.search.scope_id.replace('synthetic_conversation_m5_', 'Conversación ')}</small></span><span className={`case-status ${reviewedIds.includes(item.id) ? 'done' : ''}`}>{reviewedIds.includes(item.id) ? '✓' : '·'}</span></button>)}</div>
        {!cases.length && <div className="sidebar-empty">No hay cola cargada. Puedes revisar una pregunta libre o iniciar el servidor con un archivo de casos.</div>}
        <div className="sidebar-foot"><span className="foot-icon">◈</span><p>Las revisiones quedan en tu base local. Compartir requiere seleccionar y descargar los casos.</p></div>
      </aside>
      <main className="main-content">
        <div className="page-head"><div><p className="eyebrow">01 / Pregunta y evidencia</p><h1>{caseId ? '¿Qué recuerdos responden?' : 'Revisa una pregunta'}</h1><p className="lead">Marca solo los recuerdos que respondan. Puedes dejar el resto sin tocar y ver más candidatos si lo necesitas.</p></div><span className="stage-badge">REVISIÓN HUMANA</span></div>
        <section className="query-panel" aria-label="Consulta"><div className="panel-title"><span className="section-number">01</span><h2>Pregunta</h2><span className="panel-hint">{caseId ? `Caso ${caseId}` : 'Nueva revisión'}</span></div>
          <label htmlFor="query">Tu pregunta</label><textarea id="query" value={search.query} onChange={e => setField('query', e.target.value)} placeholder="¿Qué se acordó sobre…?" rows={2} />
          <div className="form-grid"><div><label htmlFor="scope-kind">Tipo de ámbito</label><input id="scope-kind" value={search.scope_kind} onChange={e => setField('scope_kind', e.target.value)} /></div><div><label htmlFor="scope-id">ID del ámbito</label><input id="scope-id" value={search.scope_id} onChange={e => setField('scope_id', e.target.value)} placeholder="conversation_123" /></div><div><label htmlFor="entity">Entidad (opcional)</label><input id="entity" value={search.entity_id || ''} onChange={e => setField('entity_id', e.target.value || null)} placeholder="Todas" /></div></div>
          <div className="query-controls"><div className="segmented" role="group" aria-label="Vista de candidatos"><button className={mode === 'search' ? 'selected' : ''} onClick={() => changeMode('search')}>Revisión rápida</button><button className={mode === 'scope' ? 'selected' : ''} onClick={() => changeMode('scope')}>Lista completa</button></div><button className="primary small" disabled={busy || !search.query.trim() || !search.scope_id.trim()} onClick={() => { if (mayLeave()) void load(search, mode, caseId) }}>Mostrar recuerdos <span aria-hidden="true">→</span></button></div>
        </section>
        <section id="evidence" className="evidence-section" aria-label="Recuerdos candidatos"><div className="section-heading"><div><p className="eyebrow">02 / Evidencia</p><h2>Recuerdos candidatos <span className="count">{view?.candidates.length ?? '—'}</span></h2></div>{view?.review && <span className="saved-badge">✓ Revisado {dateLabel(view.review.created_at)}</span>}</div>
          {view && <div className="review-guide" role="note"><p className="review-guide-label">Busca una respuesta a esta pregunta</p><p className="review-guide-question">{search.query}</p><p className="review-guide-help"><strong>Coincide</strong> si el recuerdo contesta la pregunta. Puedes marcar varios y guardar sin tocar los demás. <strong>No responde</strong> y <strong>Dudoso</strong> son opcionales para señalar un error concreto o una ambigüedad.</p>{mode === 'search' && <p className="review-guide-limit">Vista rápida: aparecen primero {shownCandidates.length} de {view.candidates.length} resultados de búsqueda. El orden es una sugerencia automática, no una etiqueta correcta. Este feedback no cuenta como auditoría ciega completa.</p>}{caseId && <p className="review-guide-limit">Estos casos muestran resúmenes sintéticos, no la conversación original. Aquí juzgas si un recuerdo responde, no si el hecho ocurrió.</p>}</div>}
          {view && <div className="mobile-question">{search.query}</div>}
          {caseId && reviewedIds.includes(caseId) && view && !view.review && <p className="reuse-note">Esta pregunta tiene feedback guardado en otra vista. Abre «Lista completa» si quieres consultarlo; aquí puedes crear una revisión rápida nueva.</p>}
          {view?.review && <p className="reuse-note">{view.reusable ? 'Esta revisión puede reutilizarse para la misma pregunta mientras los candidatos y revisiones sigan iguales.' : view.review.outcome === 'missing' ? 'La ausencia queda registrada para corregir la memoria; no genera una respuesta automática.' : 'La revisión queda guardada, pero una selección puede quedar fuera de los resultados de búsqueda o superar ocho recuerdos. No se aplicará automáticamente a recall.'}</p>}
          {!view && <div className="empty-state"><span className="empty-glyph">⌕</span><h3>La evidencia aparecerá aquí</h3><p>Elige una pregunta de la cola o escribe una nueva y pulsa «Mostrar recuerdos».</p></div>}
          {view && !view.candidates.length && <div className="empty-state"><span className="empty-glyph">∅</span><h3>No hay recuerdos visibles</h3><p>Comprueba el ámbito y la fecha. Puedes marcar que falta un recuerdo o que ninguno responde.</p></div>}
          <div className="candidate-grid">{shownCandidates.map((candidate, index) => <article className={`candidate ${decisions[candidate.memory_id] || ''}`} key={candidate.memory_id}><div className="candidate-top"><span className="candidate-index">{String(index + 1).padStart(2, '0')}</span><span className="revision">REV {candidate.revision}</span></div><p className="candidate-summary">{candidate.summary}</p>{candidate.conditions && <p className="condition"><span>CONDICIÓN</span>{candidate.conditions}</p>}<div className="candidate-meta"><code>{candidate.memory_id}</code>{candidate.source && <span>{candidate.source.kind} · {candidate.source.ref}</span>}</div><div className="verdict-row" role="group" aria-label={`Juicio del recuerdo ${candidate.memory_id}`}><button className={decisions[candidate.memory_id] === 'match' ? 'on match' : ''} onClick={() => mark(candidate, 'match')}>✓ Coincide</button><button className={decisions[candidate.memory_id] === 'no_match' ? 'on reject' : ''} onClick={() => mark(candidate, 'no_match')}>× No responde</button><button className={decisions[candidate.memory_id] === 'unsure' ? 'on unsure' : ''} onClick={() => mark(candidate, 'unsure')}>? Dudoso</button></div></article>)}</div>
          {partialSearch && <div className="more-candidates"><p>Si todavía no ves la respuesta, abre los siguientes resultados. No hace falta revisar los {view.candidates.length} para confirmar una coincidencia.</p><button className="ghost" onClick={() => setVisibleCount(count => Math.min(count + 5, view.candidates.length))}>Mostrar {Math.min(5, view.candidates.length - visibleCount)} más</button></div>}
        </section>
      </main>
      <aside className="decision-sidebar" aria-label="Decisión final"><div className="decision-sticky"><p className="eyebrow">03 / Tu criterio</p><h2>Decisión final</h2><p className="decision-intro">Tu corrección se guarda como una revisión. Nunca reescribe el recuerdo original.</p><div className="decision-options"><button className={outcome === 'matches' ? 'active' : ''} onClick={() => selectOutcome('matches')}><span className="option-symbol">✓</span><span><strong>Hay coincidencias</strong><small>{matches} recuerdos marcados</small></span></button><button className={outcome === 'none' ? 'active' : ''} disabled={partialSearch} onClick={() => selectOutcome('none')}><span className="option-symbol">∅</span><span><strong>Ninguno responde</strong><small>{partialSearch ? 'Muestra antes los demás resultados' : 'Hay recuerdos, pero no sirven'}</small></span></button><button className={outcome === 'missing' ? 'active' : ''} onClick={() => selectOutcome('missing')}><span className="option-symbol">＋</span><span><strong>Falta un recuerdo</strong><small>La información debería existir</small></span></button></div><p className="outcome-help">{partialSearch ? 'Puedes guardar coincidencias sin revisar todo. Para afirmar que ninguno responde, primero muestra todos los resultados.' : 'Si ningún resumen contesta, elige «Ninguno responde».'} Usa «Falta un recuerdo» solo si conoces la fuente original y sabes que se omitió un hecho.</p><label htmlFor="note">Nota de revisión <span>(opcional)</span></label><textarea id="note" rows={4} value={note} onChange={e => { setNote(e.target.value); setDirty(true) }} placeholder="Explica una ambigüedad o por qué una memoria no aplica…" /><div className="decision-footer"><button className="primary save" disabled={!view || !outcome || busy || (outcome === 'matches' && matches === 0) || (outcome === 'none' && partialSearch)} onClick={() => void save()}>{busy ? 'Procesando…' : 'Guardar revisión'} <span aria-hidden="true">→</span></button><p>La selección local puede reutilizarse si la pregunta y las revisiones siguen siendo las mismas.</p></div></div></aside>
    </div>
    {(error || message) && <div className={`toast ${error ? 'error' : ''}`} role="status"><span>{error || message}</span><button aria-label="Cerrar aviso" onClick={() => { setError(''); setMessage('') }}>×</button></div>}
    {exportRows && <div className="modal-backdrop" role="presentation" onMouseDown={e => { if (e.target === e.currentTarget) setExportRows(null) }}><div className="modal" role="dialog" aria-modal="true" aria-labelledby="export-title"><div className="modal-head"><div><p className="eyebrow">Exportación voluntaria</p><h2 id="export-title">Elige qué compartir</h2></div><button className="square-button" aria-label="Cerrar" onClick={() => setExportRows(null)}>×</button></div><p>El archivo incluirá preguntas, ámbito, resúmenes de recuerdos y tus decisiones. Revísalo antes de enviarlo a nadie. Este dashboard no sube datos automáticamente.</p><div className="export-list">{exportRows.map(row => <label key={row.id} className="export-item"><input type="checkbox" checked={chosenExports.includes(row.id)} onChange={e => setChosenExports(ids => e.target.checked ? [...ids, row.id] : ids.filter(id => id !== row.id))} /><span><strong>{row.query}</strong><small>{row.scope_id} · {row.candidates.length} recuerdos · {dateLabel(row.created_at)}</small></span></label>)}</div>{!exportRows.length && <p>No hay revisiones guardadas todavía.</p>}<div className="modal-actions"><button className="ghost" onClick={() => setExportRows(null)}>Cancelar</button><button className="primary" disabled={!chosenExports.length} onClick={download}>Descargar {chosenExports.length} seleccionadas</button></div></div></div>}
  </div>
}
