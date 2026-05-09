import { useEffect, useRef, useState } from 'react'
import {
  Activity,
  BriefcaseBusiness,
  CheckCheck,
  ChevronRight,
  ClipboardList,
  Mic,
  MicOff,
  MoonStar,
  NotebookPen,
  Settings2,
  Shield,
  Sparkles,
  Target,
} from 'lucide-react'
import { AnimatePresence, motion } from 'framer-motion'
import { Settings } from './Settings'
import {
  agentRegistry,
  createDefaultComposer,
  createDefaultSetup,
  createJournalEntry,
  deriveMirrorState,
  type AgentId,
  type ChatTurn,
  type ComposerDraft,
  type EntryImpact,
  type EntrySource,
  type JournalClassification,
  type JournalEntry,
  type MirrorSetup,
  uniqueId,
} from './mirror'

declare global {
  interface Window {
    SpeechRecognition?: new () => SpeechRecognitionLike
    webkitSpeechRecognition?: new () => SpeechRecognitionLike
  }
}

interface SpeechRecognitionLike {
  continuous: boolean
  interimResults: boolean
  lang: string
  onresult: ((event: SpeechRecognitionEventLike) => void) | null
  onstart: (() => void) | null
  onend: (() => void) | null
  onerror: (() => void) | null
  start: () => void
  stop: () => void
}

interface SpeechRecognitionEventLike {
  results: ArrayLike<{
    0: {
      transcript: string
    }
  }>
}



function completionLabel(value: number | null): string {
  return value === null ? 'Awaiting signal' : `${Math.round(value * 100)}%`
}

function metricLabel(value: number | null, suffix = ''): string {
  return value === null ? 'Awaiting signal' : `${Math.round(value)}${suffix}`
}

function timeLabel(value: string): string {
  const [hours, minutes] = value.split(':')
  const hoursNumber = Number(hours)
  const meridiem = hoursNumber >= 12 ? 'PM' : 'AM'
  const normalizedHours = hoursNumber % 12 === 0 ? 12 : hoursNumber % 12
  return `${normalizedHours}:${minutes} ${meridiem}`
}

function App() {
  const [setup, setSetup] = useState<MirrorSetup>(createDefaultSetup())
  const [entries, setEntries] = useState<JournalEntry[]>([])
  const [chat, setChat] = useState<ChatTurn[]>([])
  const [selectedAgent, setSelectedAgent] = useState<AgentId>('planner')
  const [composer, setComposer] = useState<ComposerDraft>(createDefaultComposer())
  const [isSetupOpen, setIsSetupOpen] = useState(false)
  const [isRecording, setIsRecording] = useState(false)
  const [lastInputSource, setLastInputSource] = useState<EntrySource>('text')
  const [speechError, setSpeechError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const recognitionRef = useRef<SpeechRecognitionLike | null>(null)
  const transcriptRef = useRef<EntrySource>('text')
  const [agentMetrics, setAgentMetrics] = useState({
    consistencyScore: null as number | null,
    stateGraph: null as string | null,
  })

  // Fetch history and metrics on mount
  useEffect(() => {
    const init = async () => {
      try {
        // Load memories
        const memResp = await fetch('/api/memories?category=daily')
        if (memResp.ok) {
          const memories = await memResp.json()
          const historicalEntries: JournalEntry[] = (memories as any[]).map((m) => ({
            id: m.id,
            content: m.content,
            source: 'text' as EntrySource,
            classification: 'informational' as JournalClassification,
            impact: 'none' as EntryImpact,
            deviation: false,
            majorChange: false,
            completedTask: false,
            missedTask: false,
            sleepHours: null,
            tags: [],
            createdAt: m.timestamp,
          })).sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
          
          setEntries(historicalEntries)
        }

        // Load Agent Metrics
        const scoreResp = await fetch('/api/memories/consistency_score')
        if (scoreResp.ok) {
          const data = await scoreResp.json()
          setAgentMetrics(m => ({ ...m, consistencyScore: parseInt(data.content) }))
        }
        
        const graphResp = await fetch('/api/memories/state_graph')
        if (graphResp.ok) {
           const data = await graphResp.json()
           setAgentMetrics(m => ({ ...m, stateGraph: data.content }))
        }
      } catch (e) {
        console.error('Failed to initialize Mirror state', e)
      }
    }
    init()
  }, [])

  const derivedBase = deriveMirrorState(setup, entries)
  const derived = {
    ...derivedBase,
    consistencyScore: agentMetrics.consistencyScore ?? derivedBase.consistencyScore,
    recentBehavioralSummary: agentMetrics.stateGraph 
       ? [agentMetrics.stateGraph, ...derivedBase.recentBehavioralSummary]
       : derivedBase.recentBehavioralSummary
  }
  const activeAgent = agentRegistry.find((agent) => agent.id === selectedAgent) ?? agentRegistry[0]
  const agentChat = chat.filter((turn) => turn.agentId === selectedAgent)
  const latestEntry = entries[0] ?? null
  const deviationsToday = entries.filter((entry) => entry.deviation).length


  useEffect(() => {
    return () => {
      recognitionRef.current?.stop()
    }
  }, [])

  const speechSupported =
    typeof window !== 'undefined' &&
    Boolean(window.SpeechRecognition || window.webkitSpeechRecognition)

  const handleToggleRecording = () => {
    if (!speechSupported) {
      setSpeechError('Speech input is not available in this browser.')
      return
    }

    if (recognitionRef.current && isRecording) {
      recognitionRef.current.stop()
      return
    }

    const RecognitionCtor = window.SpeechRecognition ?? window.webkitSpeechRecognition
    if (!RecognitionCtor) {
      setSpeechError('Speech input is not available in this browser.')
      return
    }

    const recognition = new RecognitionCtor()
    recognition.continuous = false
    recognition.interimResults = false
    recognition.lang = 'en-US'
    recognition.onstart = () => {
      transcriptRef.current = 'speech'
      setSpeechError(null)
      setIsRecording(true)
    }
    recognition.onresult = (event) => {
      const transcript = Array.from(event.results)
        .map((result) => result[0]?.transcript ?? '')
        .join(' ')
        .trim()

      if (!transcript) {
        return
      }

      setComposer((current) => ({
        ...current,
        text: current.text ? `${current.text} ${transcript}` : transcript,
      }))
      setLastInputSource('speech')
    }
    recognition.onerror = () => {
      setSpeechError('Speech capture failed. You can still type the entry manually.')
      setIsRecording(false)
      recognitionRef.current = null
    }
    recognition.onend = () => {
      setIsRecording(false)
      recognitionRef.current = null
    }

    recognitionRef.current = recognition
    recognition.start()
  }
  const handleSubmit = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    if (!composer.text.trim() || isLoading) {
      return
    }

    const source = lastInputSource === 'speech' ? 'speech' : 'text'
    const entry = createJournalEntry(composer, source)
    const nextEntries = [entry, ...entries]
    const userTurn: ChatTurn = {
      id: uniqueId('chat'),
      agentId: selectedAgent,
      role: 'user',
      content: composer.text.trim(),
      createdAt: new Date().toISOString(),
    }

    setEntries(nextEntries)
    setChat((current) => [...current, userTurn])
    setComposer(createDefaultComposer())
    setLastInputSource('text')
    setIsLoading(true)

    try {
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          message: userTurn.content,
          agent_id: selectedAgent,
        }),
      })

      if (!response.ok) {
        throw new Error('Failed to reach Mirror brain.')
      }

      const data = await response.json()
      const assistantTurn: ChatTurn = {
        id: uniqueId('chat'),
        agentId: selectedAgent,
        role: 'assistant',
        content: data.response || 'No response captured.',
        createdAt: new Date().toISOString(),
      }

      setChat((current) => [...current, assistantTurn])
    } catch (error) {
      const errorTurn: ChatTurn = {
        id: uniqueId('chat'),
        agentId: selectedAgent,
        role: 'assistant',
        content: `Error: ${error instanceof Error ? error.message : 'Unknown connection error.'}`,
        createdAt: new Date().toISOString(),
      }
      setChat((current) => [...current, errorTurn])
    } finally {
      setIsLoading(false)
    }
  }

  const renderAgentOutput = () => {
    if (selectedAgent === 'planner') {
      return (
        <div className="agent-stack">
          <section className="agent-panel">
            <div className="panel-head">
              <div>
                <div className="panel-eyebrow">Daily plan</div>
                <h2 className="panel-title">Time structure for today</h2>
              </div>
              <ClipboardList size={18} className="panel-icon" />
            </div>
            <div className="timeline-list">
              {derived.todayPlan.map((block) => (
                <div key={`${block.start}-${block.title}`} className={`timeline-item lane-${block.lane}`}>
                  <div className="timeline-time">
                    {timeLabel(block.start)}
                    <span>→</span>
                    {timeLabel(block.end)}
                  </div>
                  <div>
                    <div className="timeline-title">{block.title}</div>
                    <div className="timeline-note">{block.note}</div>
                  </div>
                </div>
              ))}
            </div>
          </section>

          <section className="split-grid">
            <article className="agent-panel">
              <div className="panel-head">
                <div>
                  <div className="panel-eyebrow">Priority stack</div>
                  <h2 className="panel-title">What matters next</h2>
                </div>
                <Target size={18} className="panel-icon" />
              </div>
              <div className="bullet-list">
                {derived.priorities.map((priority) => (
                  <div key={priority} className="bullet-item">
                    <ChevronRight size={14} />
                    <span>{priority}</span>
                  </div>
                ))}
              </div>
            </article>

            <article className="agent-panel">
              <div className="panel-head">
                <div>
                  <div className="panel-eyebrow">Deviation watch</div>
                  <h2 className="panel-title">Planner alerts</h2>
                </div>
                <Activity size={18} className="panel-icon" />
              </div>
              <div className="bullet-list">
                {derived.plannerAlerts.map((alert) => (
                  <div key={alert} className="bullet-item">
                    <ChevronRight size={14} />
                    <span>{alert}</span>
                  </div>
                ))}
              </div>
            </article>
          </section>
        </div>
      )
    }

    if (selectedAgent === 'journal') {
      return (
        <div className="agent-stack">
          <section className="split-grid">
            <article className="agent-panel compact-panel">
              <div className="metric-card-label">Informational</div>
              <div className="metric-card-value">{derived.classificationCounts.informational}</div>
              <div className="metric-card-note">Logged facts and status updates</div>
            </article>
            <article className="agent-panel compact-panel">
              <div className="metric-card-label">Behavioral</div>
              <div className="metric-card-value">{derived.classificationCounts.behavioral}</div>
              <div className="metric-card-note">Deviation from plan or intent</div>
            </article>
            <article className="agent-panel compact-panel">
              <div className="metric-card-label">Critical</div>
              <div className="metric-card-value">{derived.classificationCounts.critical}</div>
              <div className="metric-card-note">Major changes that require replanning</div>
            </article>
          </section>

          <section className="agent-panel">
            <div className="panel-head">
              <div>
                <div className="panel-eyebrow">Capture stream</div>
                <h2 className="panel-title">Primary input engine</h2>
              </div>
              <NotebookPen size={18} className="panel-icon" />
            </div>
            <div className="entry-feed">
              {entries.length > 0 ? (
                entries.slice(0, 8).map((entry) => (
                  <article key={entry.id} className="entry-card">
                    <div className="entry-meta">
                      <span className={`entry-badge is-${entry.classification}`}>{entry.classification}</span>
                      <span>{new Date(entry.createdAt).toLocaleString()}</span>
                      <span>{entry.source}</span>
                    </div>
                    <div className="entry-content">{entry.content}</div>
                  </article>
                ))
              ) : (
                <div className="empty-panel">
                  Journal Agent is waiting for the first real input. Text and speech both land here.
                </div>
              )}
            </div>
          </section>
        </div>
      )
    }

    if (selectedAgent === 'career') {
      return (
        <div className="agent-stack">
          <section className="split-grid">
            <article className="agent-panel">
              <div className="panel-head">
                <div>
                  <div className="panel-eyebrow">Target roles</div>
                  <h2 className="panel-title">Direction of travel</h2>
                </div>
                <BriefcaseBusiness size={18} className="panel-icon" />
              </div>
              <div className="tag-cloud">
                {setup.targetRoles.length > 0 ? (
                  setup.targetRoles.map((role) => (
                    <span key={role} className="tag-chip">
                      {role}
                    </span>
                  ))
                ) : (
                  <div className="empty-panel">Add target roles in setup to activate career alignment.</div>
                )}
              </div>
            </article>

            <article className="agent-panel">
              <div className="panel-head">
                <div>
                  <div className="panel-eyebrow">Gap analysis</div>
                  <h2 className="panel-title">Missing or weak areas</h2>
                </div>
                <Target size={18} className="panel-icon" />
              </div>
              <div className="bullet-list">
                {derived.gapSkills.length > 0 ? (
                  derived.gapSkills.map((gap) => (
                    <div key={gap} className="bullet-item">
                      <ChevronRight size={14} />
                      <span>{gap}</span>
                    </div>
                  ))
                ) : (
                  <div className="empty-panel">No gap can be computed until required skills are filled in.</div>
                )}
              </div>
            </article>
          </section>

          <section className="split-grid">
            <article className="agent-panel">
              <div className="panel-head">
                <div>
                  <div className="panel-eyebrow">Action mapping</div>
                  <h2 className="panel-title">If you want X, do Y</h2>
                </div>
                <Sparkles size={18} className="panel-icon" />
              </div>
              <div className="bullet-list">
                {derived.careerActions.length > 0 ? (
                  derived.careerActions.map((action) => (
                    <div key={action} className="bullet-item">
                      <ChevronRight size={14} />
                      <span>{action}</span>
                    </div>
                  ))
                ) : (
                  <div className="empty-panel">Career actions appear once both target roles and required skills exist.</div>
                )}
              </div>
            </article>

            <article className="agent-panel">
              <div className="panel-head">
                <div>
                  <div className="panel-eyebrow">Behavioral integration</div>
                  <h2 className="panel-title">Alignment warnings</h2>
                </div>
                <Shield size={18} className="panel-icon" />
              </div>
              <div className="bullet-list">
                {derived.alignmentWarnings.length > 0 ? (
                  derived.alignmentWarnings.map((warning) => (
                    <div key={warning} className="bullet-item">
                      <ChevronRight size={14} />
                      <span>{warning}</span>
                    </div>
                  ))
                ) : (
                  <div className="empty-panel">
                    No mismatch is currently visible between configured goals and captured behavior.
                  </div>
                )}
              </div>
            </article>
          </section>
        </div>
      )
    }

    return (
      <div className="agent-stack">
        <section className="split-grid">
          <article className="agent-panel compact-panel">
            <div className="metric-card-label">Consistency score</div>
            <div className="metric-card-value">{metricLabel(derived.consistencyScore)}</div>
            <div className="metric-card-note">Derived from deviation, completion, and sleep signals</div>
          </article>
          <article className="agent-panel compact-panel">
            <div className="metric-card-label">Completion rate</div>
            <div className="metric-card-value">{completionLabel(derived.completionRate)}</div>
            <div className="metric-card-note">Planned completions vs reported misses</div>
          </article>
          <article className="agent-panel compact-panel">
            <div className="metric-card-label">Average sleep</div>
            <div className="metric-card-value">{metricLabel(derived.averageSleep, 'h')}</div>
            <div className="metric-card-note">Only from explicit sleep journal entries</div>
          </article>
        </section>

        <section className="split-grid">
          <article className="agent-panel">
            <div className="panel-head">
              <div>
                <div className="panel-eyebrow">Pattern detection</div>
                <h2 className="panel-title">Behavior drift</h2>
              </div>
              <Activity size={18} className="panel-icon" />
            </div>
            <div className="bullet-list">
              {derived.patternWarnings.length > 0 ? (
                derived.patternWarnings.map((warning) => (
                  <div key={warning} className="bullet-item">
                    <ChevronRight size={14} />
                    <span>{warning}</span>
                  </div>
                ))
              ) : (
                <div className="empty-panel">No negative pattern is visible yet.</div>
              )}
            </div>
          </article>

          <article className="agent-panel">
            <div className="panel-head">
              <div>
                <div className="panel-eyebrow">Latest reality check</div>
                <h2 className="panel-title">Recent behavior summary</h2>
              </div>
              <CheckCheck size={18} className="panel-icon" />
            </div>
            <div className="bullet-list">
              {derived.recentBehavioralSummary.length > 0 ? (
                derived.recentBehavioralSummary.map((item) => (
                  <div key={item} className="bullet-item">
                    <ChevronRight size={14} />
                    <span>{item}</span>
                  </div>
                ))
              ) : (
                <div className="empty-panel">No journal entries have been captured yet.</div>
              )}
            </div>
          </article>
        </section>
      </div>
    )
  }

  return (
    <div className="mirror-shell">
      <div className="mirror-grid" />

      <header className="mirror-header">
        <div className="brand-block">
          <div className="brand-badge">
            <Shield size={18} />
          </div>
          <div>
            <div className="brand-title">Mirror</div>
            <div className="brand-subtitle">Autonomous Personal Intelligence System</div>
          </div>
        </div>

        <div className="header-actions">
          <button type="button" className="secondary-button" onClick={() => setIsSetupOpen(true)}>
            <Settings2 size={14} />
            System Setup
          </button>
        </div>
      </header>

      <section className="hero-panel">
        <div>
          <div className="hero-eyebrow">Core objective</div>
          <h1 className="hero-title">
            Daily behavior → time usage → career outcomes, reflected back as measurable reality.
          </h1>
          <p className="hero-copy">
            Journal Agent stays always on. Planner reacts to deviations without overwriting them.
            Career Agent keeps the day pointed at target roles. Health Agent measures consistency
            instead of guessing it.
          </p>
        </div>

        <div className="hero-metrics">
          <div className="metric-tile">
            <span className="metric-kicker">Journal entries</span>
            <strong>{entries.length}</strong>
          </div>
          <div className="metric-tile">
            <span className="metric-kicker">Behavior drift</span>
            <strong>{deviationsToday}</strong>
          </div>
          <div className="metric-tile">
            <span className="metric-kicker">Open career gaps</span>
            <strong>{derived.gapSkills.length}</strong>
          </div>
          <div className="metric-tile">
            <span className="metric-kicker">Consistency</span>
            <strong>{metricLabel(derived.consistencyScore)}</strong>
          </div>
        </div>
      </section>

      <main className="mirror-layout">
        <aside className="agent-sidebar">
          <div className="sidebar-panel">
            <div className="sidebar-eyebrow">Agents</div>
            <div className="sidebar-title">Exactly four, no overlap</div>
            <div className="agent-nav">
              {agentRegistry.map((agent) => (
                <button
                  key={agent.id}
                  type="button"
                  className={`agent-nav-item ${selectedAgent === agent.id ? 'is-active' : ''}`}
                  onClick={() => setSelectedAgent(agent.id)}
                >
                  <div className="agent-nav-title">{agent.label}</div>
                  <div className="agent-nav-role">{agent.role}</div>
                  <div className="agent-nav-summary">{agent.summary}</div>
                </button>
              ))}
            </div>

            <div className="sidebar-rule">
              <div className="sidebar-rule-title">Critical system rules</div>
              <div className="rule-item">Do not overwrite deviations.</div>
              <div className="rule-item">Always track intention vs action.</div>
              <div className="rule-item">Only use available input.</div>
            </div>
          </div>
        </aside>

        <section className="main-column">
          <AnimatePresence mode="wait">
            <motion.div
              key={selectedAgent}
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -12 }}
              transition={{ duration: 0.16 }}
            >
              <section className="selected-agent-banner">
                <div>
                  <div className="panel-eyebrow">{activeAgent.role}</div>
                  <h2 className="panel-title">{activeAgent.label}</h2>
                  <p className="panel-copy">{activeAgent.summary}</p>
                </div>
                <div className="agent-signal">
                  <span className={`signal-dot ${latestEntry?.majorChange ? 'is-critical' : latestEntry?.deviation ? 'is-warning' : 'is-stable'}`} />
                  {latestEntry?.majorChange
                    ? 'Critical change detected'
                    : latestEntry?.deviation
                      ? 'Deviation detected'
                      : 'Stable input stream'}
                </div>
              </section>

              {renderAgentOutput()}
            </motion.div>
          </AnimatePresence>

          <section className="chat-layout">
            <article className="chat-panel">
              <div className="panel-head">
                <div>
                  <div className="panel-eyebrow">Agent chat</div>
                  <h2 className="panel-title">Shared input, agent-specific response</h2>
                </div>
                <NotebookPen size={18} className="panel-icon" />
              </div>

              <div className="chat-stream">
                {agentChat.length > 0 ? (
                  agentChat.map((turn) => (
                    <div key={turn.id} className={`chat-bubble is-${turn.role}`}>
                      <div className="chat-meta">
                        <span>{turn.role}</span>
                        <span>{new Date(turn.createdAt).toLocaleTimeString()}</span>
                      </div>
                      <div>{turn.content}</div>
                    </div>
                  ))
                ) : (
                  <div className="empty-panel">
                    No chat turns for this agent yet. The Journal Agent still records any input you
                    send below.
                  </div>
                )}
              </div>
            </article>

            <article className="composer-panel">
              <div className="panel-head">
                <div>
                  <div className="panel-eyebrow">Input layer</div>
                  <h2 className="panel-title">Journal Agent is always active</h2>
                </div>
                <Sparkles size={18} className="panel-icon" />
              </div>

              <form className="composer-form" onSubmit={handleSubmit}>
                <div className="composer-row">
                  <label className="mini-field">
                    <span>Classification</span>
                    <select
                      className="control"
                      value={composer.classification}
                      onChange={(event) =>
                        setComposer((current) => ({
                          ...current,
                          classification: event.target.value as ComposerDraft['classification'],
                        }))
                      }
                    >
                      <option value="auto">Auto classify</option>
                      <option value="informational">Informational</option>
                      <option value="behavioral">Behavioral</option>
                      <option value="critical">Critical</option>
                    </select>
                  </label>

                  <label className="mini-field">
                    <span>Impact</span>
                    <select
                      className="control"
                      value={composer.impact}
                      onChange={(event) =>
                        setComposer((current) => ({
                          ...current,
                          impact: event.target.value as ComposerDraft['impact'],
                        }))
                      }
                    >
                      <option value="none">No explicit impact</option>
                      <option value="completed_task">Completed planned task</option>
                      <option value="missed_task">Missed planned task</option>
                      <option value="major_change">Major change</option>
                    </select>
                  </label>

                  <label className="mini-field">
                    <span>Sleep (optional)</span>
                    <input
                      className="control"
                      type="number"
                      min="0"
                      step="0.5"
                      value={composer.sleepHours}
                      onChange={(event) =>
                        setComposer((current) => ({
                          ...current,
                          sleepHours: event.target.value,
                        }))
                      }
                      placeholder="7.5"
                    />
                  </label>
                </div>

                <textarea
                  className="control control-textarea composer-textarea"
                  rows={6}
                  value={composer.text}
                  onChange={(event) => {
                    setComposer((current) => ({
                      ...current,
                      text: event.target.value,
                    }))
                    setLastInputSource('text')
                  }}
                  placeholder={`Send a real-world update to ${activeAgent.label}. Example: "I skipped the gym and I am not going to office today."`}
                />

                <div className="composer-footer">
                  <div className="composer-notes">
                    <div className="note-pill">
                      <span className="note-label">Current route</span>
                      <span>{activeAgent.label}</span>
                    </div>
                    <div className="note-pill">
                      <span className="note-label">Input source</span>
                      <span>{lastInputSource}</span>
                    </div>
                    {speechError ? <div className="note-warning">{speechError}</div> : null}
                  </div>

                  <div className="composer-actions">
                    <button
                      type="button"
                      className="secondary-button"
                      onClick={handleToggleRecording}
                    >
                      {isRecording ? <MicOff size={14} /> : <Mic size={14} />}
                      {isRecording ? 'Stop speech' : 'Use speech'}
                    </button>
                    <button type="submit" className="primary-button" disabled={isLoading}>
                      {isLoading ? (
                        <div className="button-spinner" />
                      ) : (
                        <NotebookPen size={14} />
                      )}
                      {isLoading ? 'Processing...' : 'Log to system'}
                    </button>
                  </div>
                </div>
              </form>
            </article>
          </section>
        </section>

        <aside className="insight-rail">
          <div className="rail-panel">
            <div className="sidebar-eyebrow">Operating state</div>
            <div className="sidebar-title">Measurable reality</div>

            <div className="rail-stack">
              <div className="rail-card">
                <div className="rail-card-label">Last input</div>
                <div className="rail-card-value">
                  {latestEntry ? latestEntry.content : 'No input captured yet'}
                </div>
              </div>

              <div className="rail-card">
                <div className="rail-card-label">Next planner block</div>
                <div className="rail-card-value">
                  {derived.todayPlan[0]
                    ? `${derived.todayPlan[0].title} · ${timeLabel(derived.todayPlan[0].start)}`
                    : 'Awaiting setup'}
                </div>
              </div>

              <div className="rail-card">
                <div className="rail-card-label">Career focus</div>
                <div className="rail-card-value">
                  {derived.gapSkills[0] ?? 'Add required skills to compute the first gap'}
                </div>
              </div>

              <div className="rail-card">
                <div className="rail-card-label">Sleep target</div>
                <div className="rail-card-value">
                  <MoonStar size={14} />
                  {setup.sleepTargetHours}h target / {metricLabel(derived.averageSleep, 'h')} actual
                </div>
              </div>
            </div>
          </div>
        </aside>
      </main>

      {isSetupOpen ? (
        <Settings setup={setup} onChange={setSetup} onClose={() => setIsSetupOpen(false)} />
      ) : null}
    </div>
  )
}

export default App
