export type AgentId = 'planner' | 'journal' | 'career' | 'health'
export type EntrySource = 'text' | 'speech'
export type JournalClassification = 'informational' | 'behavioral' | 'critical'
export type EntryImpact = 'none' | 'completed_task' | 'missed_task' | 'major_change'

export interface MirrorSetup {
  targetRoles: string[]
  currentSkills: string[]
  requiredSkills: string[]
  weakAreas: string[]
  resumeNotes: string
  portfolioLinks: string[]
  supportingDocuments: string[]
  calendarNotes: string
  timetableNotes: string
  dayStart: string
  workStart: string
  workEnd: string
  dayEnd: string
  skillBlockHours: number
  healthBlockMinutes: number
  breakMinutes: number
  sleepTargetHours: number
  workoutTargetPerWeek: number
  officeDays: string[]
}

export interface JournalEntry {
  id: string
  createdAt: string
  content: string
  source: EntrySource
  classification: JournalClassification
  impact: EntryImpact
  deviation: boolean
  majorChange: boolean
  completedTask: boolean
  missedTask: boolean
  sleepHours: number | null
  tags: string[]
}

export interface ChatTurn {
  id: string
  agentId: AgentId
  role: 'user' | 'assistant'
  content: string
  createdAt: string
}

export interface ComposerDraft {
  text: string
  classification: 'auto' | JournalClassification
  impact: EntryImpact
  sleepHours: string
}

export interface PlanBlock {
  start: string
  end: string
  title: string
  lane: 'work' | 'skill' | 'health' | 'reflection' | 'alert'
  note: string
}

export interface DerivedMirrorState {
  todayPlan: PlanBlock[]
  priorities: string[]
  gapSkills: string[]
  alignmentWarnings: string[]
  plannerAlerts: string[]
  classificationCounts: Record<JournalClassification, number>
  recentBehavioralSummary: string[]
  completionRate: number | null
  consistencyScore: number | null
  patternWarnings: string[]
  averageSleep: number | null
  careerActions: string[]
}

export const agentRegistry: Array<{
  id: AgentId
  label: string
  role: string
  summary: string
}> = [
  {
    id: 'planner',
    label: 'Planner Agent',
    role: 'Day structure',
    summary: 'Controls time blocks, priorities, and schedule corrections when life changes.',
  },
  {
    id: 'journal',
    label: 'Journal Agent',
    role: 'Primary input engine',
    summary: 'Captures text and speech, classifies reality, and feeds the rest of the system.',
  },
  {
    id: 'career',
    label: 'Career Agent',
    role: 'Long-term alignment',
    summary: 'Maps daily behavior to target roles, gaps, and required proof of work.',
  },
  {
    id: 'health',
    label: 'Health / Behavior Agent',
    role: 'Consistency tracking',
    summary: 'Monitors sleep, completion trends, and behavioral drift before it compounds.',
  },
]

export function createDefaultSetup(): MirrorSetup {
  return {
    targetRoles: [],
    currentSkills: [],
    requiredSkills: [],
    weakAreas: [],
    resumeNotes: '',
    portfolioLinks: [],
    supportingDocuments: [],
    calendarNotes: '',
    timetableNotes: '',
    dayStart: '07:00',
    workStart: '09:00',
    workEnd: '17:00',
    dayEnd: '22:00',
    skillBlockHours: 2,
    healthBlockMinutes: 45,
    breakMinutes: 15,
    sleepTargetHours: 8,
    workoutTargetPerWeek: 4,
    officeDays: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri'],
  }
}

export function createDefaultComposer(): ComposerDraft {
  return {
    text: '',
    classification: 'auto',
    impact: 'none',
    sleepHours: '',
  }
}

export function parseLineList(value: string): string[] {
  return value
    .split('\n')
    .map((entry) => entry.trim())
    .filter(Boolean)
}

export function formatLineList(values: string[]): string {
  return values.join('\n')
}

export function uniqueId(prefix: string): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return `${prefix}-${crypto.randomUUID()}`
  }

  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

function lowerList(values: string[]): string[] {
  return values.map((value) => value.trim().toLowerCase())
}

function timeToMinutes(value: string): number {
  const [hours, minutes] = value.split(':').map(Number)
  return hours * 60 + minutes
}

function minutesToTime(value: number): string {
  const normalized = Math.max(0, value)
  const hours = Math.floor(normalized / 60)
  const minutes = normalized % 60
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`
}

function classificationFromContent(content: string, impact: EntryImpact): JournalClassification {
  if (impact === 'major_change') {
    return 'critical'
  }

  if (impact === 'missed_task') {
    return 'behavioral'
  }

  const normalized = content.toLowerCase()
  const criticalSignals = [
    'not going to office',
    'not going into office',
    'off today',
    'sick',
    'travel',
    'urgent',
    'emergency',
    'reschedule',
    'cannot work',
    "can't work",
  ]
  const behavioralSignals = [
    'skipped',
    'missed',
    'late',
    'procrastinated',
    'did not',
    "didn't",
    'could not',
    "couldn't",
    'failed to',
  ]

  if (criticalSignals.some((signal) => normalized.includes(signal))) {
    return 'critical'
  }

  if (behavioralSignals.some((signal) => normalized.includes(signal))) {
    return 'behavioral'
  }

  return 'informational'
}

export function createJournalEntry(
  draft: ComposerDraft,
  source: EntrySource,
): JournalEntry {
  const trimmed = draft.text.trim()
  const resolvedImpact = draft.impact
  const resolvedClassification =
    draft.classification === 'auto'
      ? classificationFromContent(trimmed, resolvedImpact)
      : draft.classification
  const normalized = trimmed.toLowerCase()
  const completedTask =
    resolvedImpact === 'completed_task' ||
    normalized.includes('completed') ||
    normalized.includes('finished') ||
    normalized.includes('done')
  const missedTask =
    resolvedImpact === 'missed_task' ||
    normalized.includes('missed') ||
    normalized.includes('skipped')
  const majorChange =
    resolvedImpact === 'major_change' || resolvedClassification === 'critical'
  const deviation = missedTask || resolvedClassification === 'behavioral'
  const sleepHours =
    draft.sleepHours.trim() === '' ? null : Number.parseFloat(draft.sleepHours.trim())

  return {
    id: uniqueId('entry'),
    createdAt: new Date().toISOString(),
    content: trimmed,
    source,
    classification: resolvedClassification,
    impact: resolvedImpact,
    deviation,
    majorChange,
    completedTask,
    missedTask,
    sleepHours: Number.isFinite(sleepHours) ? sleepHours : null,
    tags: [
      resolvedClassification,
      ...(majorChange ? ['major-change'] : []),
      ...(deviation ? ['deviation'] : []),
      ...(sleepHours !== null ? ['sleep'] : []),
    ],
  }
}

function entryIsToday(entry: JournalEntry): boolean {
  const created = new Date(entry.createdAt)
  const now = new Date()
  return created.toDateString() === now.toDateString()
}

function average(values: number[]): number | null {
  if (values.length === 0) {
    return null
  }

  return values.reduce((sum, value) => sum + value, 0) / values.length
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value))
}

function buildCareerActions(targetRoles: string[], gapSkills: string[]): string[] {
  if (targetRoles.length === 0 && gapSkills.length === 0) {
    return []
  }

  return gapSkills.slice(0, 4).map((skill) => {
    const normalized = skill.toLowerCase()

    if (normalized.includes('dsa') || normalized.includes('algorithm')) {
      return `Want ${targetRoles.join(', ') || 'the target role'} -> schedule recurring DSA practice for ${skill}.`
    }

    if (normalized.includes('project') || normalized.includes('portfolio')) {
      return `Want ${targetRoles.join(', ') || 'the target role'} -> ship visible proof of work for ${skill}.`
    }

    if (normalized.includes('system design')) {
      return `Want ${targetRoles.join(', ') || 'the target role'} -> review one system design case and document it.`
    }

    return `Want ${targetRoles.join(', ') || 'the target role'} -> create a focused practice block for ${skill}.`
  })
}

function buildPlannerAlerts(entries: JournalEntry[]): string[] {
  const alerts: string[] = []
  const latestMajorChange = entries.find((entry) => entryIsToday(entry) && entry.majorChange)
  const latestDeviation = entries.find((entry) => entryIsToday(entry) && entry.deviation)

  if (latestMajorChange) {
    alerts.push(`Major change recorded: "${latestMajorChange.content}"`)
  }

  if (latestDeviation) {
    alerts.push(`Behavioral deviation recorded: "${latestDeviation.content}"`)
  }

  if (alerts.length === 0) {
    alerts.push('No plan-breaking changes have been recorded today.')
  }

  return alerts
}

function buildPlan(setup: MirrorSetup, entries: JournalEntry[], gapSkills: string[]): PlanBlock[] {
  const todayEntries = entries.filter(entryIsToday)
  const start = timeToMinutes(setup.dayStart)
  const workStart = timeToMinutes(setup.workStart)
  const workEnd = timeToMinutes(setup.workEnd)
  const end = timeToMinutes(setup.dayEnd)
  const healthDuration = setup.healthBlockMinutes
  const skillDuration = Math.round(setup.skillBlockHours * 60)
  const blocks: PlanBlock[] = []
  const hasRecoverySignal = todayEntries.some((entry) =>
    ['sick', 'cannot work', "can't work", 'recovery'].some((signal) =>
      entry.content.toLowerCase().includes(signal),
    ),
  )
  const remoteSignal = todayEntries.some((entry) =>
    ['not going to office', 'not going into office', 'work from home', 'wfh'].some((signal) =>
      entry.content.toLowerCase().includes(signal),
    ),
  )

  if (hasRecoverySignal) {
    return [
      {
        start: setup.dayStart,
        end: setup.dayEnd,
        title: 'Recovery / minimum viable commitments',
        lane: 'alert',
        note: 'Planner paused the normal structure because today includes a health or recovery change.',
      },
    ]
  }

  if (workStart > start + 45 && gapSkills.length > 0) {
    blocks.push({
      start: minutesToTime(start),
      end: setup.workStart,
      title: 'Career warm-up',
      lane: 'skill',
      note: 'Use this block to close the highest-priority role gap before the workday starts.',
    })
  }

  blocks.push({
    start: setup.workStart,
    end: setup.workEnd,
    title: remoteSignal ? 'Work / study (adjusted for plan change)' : 'Work / study block',
    lane: 'work',
    note: remoteSignal
      ? 'A critical journal update changed the context for today, so keep this block flexible.'
      : 'Anchor block reserved for core work, study, or office responsibilities.',
  })

  const skillStart = workEnd + setup.breakMinutes
  const skillEnd = Math.min(skillStart + skillDuration, end - healthDuration - 30)
  if (skillDuration > 0 && skillEnd > skillStart) {
    blocks.push({
      start: minutesToTime(skillStart),
      end: minutesToTime(skillEnd),
      title: 'Career development block',
      lane: 'skill',
      note:
        gapSkills.length > 0
          ? `Prioritize ${gapSkills.slice(0, 2).join(' and ')}.`
          : 'Keep this block for structured skill-building or portfolio work.',
    })
  }

  const healthStart = Math.min(skillEnd + setup.breakMinutes, end - healthDuration - 30)
  const healthEnd = Math.min(healthStart + healthDuration, end - 30)
  if (healthEnd > healthStart) {
    blocks.push({
      start: minutesToTime(healthStart),
      end: minutesToTime(healthEnd),
      title: 'Health / recovery',
      lane: 'health',
      note: 'Protect this block for movement, meals, sleep preparation, or recovery.',
    })
  }

  blocks.push({
    start: minutesToTime(Math.max(end - 30, healthEnd)),
    end: setup.dayEnd,
    title: 'Night reflection',
    lane: 'reflection',
    note: 'Compare intention vs action and log what actually happened.',
  })

  return blocks
}

export function deriveMirrorState(
  setup: MirrorSetup,
  entries: JournalEntry[],
): DerivedMirrorState {
  const classificationCounts: Record<JournalClassification, number> = {
    informational: 0,
    behavioral: 0,
    critical: 0,
  }

  for (const entry of entries) {
    classificationCounts[entry.classification] += 1
  }

  const normalizedCurrentSkills = new Set(lowerList(setup.currentSkills))
  const gapSkills = setup.requiredSkills.filter(
    (skill) => !normalizedCurrentSkills.has(skill.trim().toLowerCase()),
  )
  const careerActions = buildCareerActions(setup.targetRoles, gapSkills)
  const todayEntries = entries.filter(entryIsToday)
  const recentEntries = entries.slice(0, 7)
  const recentBehavioralSummary = recentEntries.slice(0, 4).map((entry) => {
    const prefix =
      entry.classification === 'critical'
        ? 'Critical'
        : entry.classification === 'behavioral'
          ? 'Deviation'
          : 'Info'

    return `${prefix}: ${entry.content}`
  })
  const completedCount = entries.filter((entry) => entry.completedTask).length
  const missedCount = entries.filter((entry) => entry.missedTask || entry.deviation).length
  const completionRate =
    completedCount + missedCount === 0 ? null : completedCount / (completedCount + missedCount)
  const sleepSamples = entries
    .map((entry) => entry.sleepHours)
    .filter((value): value is number => value !== null && Number.isFinite(value))
  const averageSleep = average(sleepSamples)
  const sleepPenalty =
    averageSleep === null ? 0 : Math.abs(averageSleep - setup.sleepTargetHours) * 8
  const consistencyScore = clamp(
    Math.round(
      82 +
        completedCount * 6 -
        missedCount * 10 -
        sleepPenalty -
        classificationCounts.critical * 8,
    ),
    0,
    100,
  )
  const alignmentWarnings: string[] = []

  if (gapSkills.length > 0 && setup.skillBlockHours <= 0) {
    alignmentWarnings.push('Career gaps exist, but no skill-development block is configured.')
  }

  if (gapSkills.length > 0 && todayEntries.some((entry) => entry.missedTask)) {
    alignmentWarnings.push('Today includes missed planned work while career gaps are still open.')
  }

  if (setup.targetRoles.length === 0) {
    alignmentWarnings.push('Target roles are missing, so long-term alignment cannot be measured yet.')
  }

  const patternWarnings: string[] = []
  if (missedCount >= 3) {
    patternWarnings.push(`You have logged ${missedCount} missed or drift events so far.`)
  }
  if (averageSleep !== null && averageSleep < setup.sleepTargetHours - 1) {
    patternWarnings.push('Recent sleep is below the target baseline.')
  }
  if (classificationCounts.critical > 0) {
    patternWarnings.push('Critical changes are appearing in the journal and should trigger plan review.')
  }
  if (todayEntries.length === 0) {
    patternWarnings.push('No journal entry has been recorded today yet.')
  }

  const priorities: string[] = []
  if (gapSkills.length > 0) {
    priorities.push(`Close the highest-value gap: ${gapSkills[0]}.`)
  }
  if (todayEntries.some((entry) => entry.missedTask)) {
    priorities.push('Recover one missed task before the day ends.')
  }
  if (completionRate !== null && completionRate < 0.5) {
    priorities.push('Reduce drift by finishing one planned action before logging anything new.')
  }
  if (priorities.length === 0) {
    priorities.push('Capture the first real-world journal check-in for today.')
  }

  return {
    todayPlan: buildPlan(setup, entries, gapSkills),
    priorities,
    gapSkills,
    alignmentWarnings,
    plannerAlerts: buildPlannerAlerts(entries),
    classificationCounts,
    recentBehavioralSummary,
    completionRate,
    consistencyScore,
    patternWarnings,
    averageSleep,
    careerActions,
  }
}

export function buildAgentReply(
  agentId: AgentId,
  setup: MirrorSetup,
  derived: DerivedMirrorState,
  entry: JournalEntry,
): string {
  if (agentId === 'journal') {
    return `Logged as ${entry.classification}. ${entry.majorChange ? 'Planner adjustment required.' : entry.deviation ? 'This counts as a deviation from plan.' : 'Reality captured without changing the plan.'}`
  }

  if (agentId === 'planner') {
    const nextBlock = derived.todayPlan[0]
    return entry.majorChange
      ? `Critical change recorded. Do not overwrite the plan silently. Re-anchor around "${nextBlock?.title ?? 'today'}" and recover one priority at a time.`
      : `Planner sees ${derived.priorities[0]} Next block: ${nextBlock?.title ?? 'capture more input'} (${nextBlock?.start ?? '--'}-${nextBlock?.end ?? '--'}).`
  }

  if (agentId === 'career') {
    if (setup.targetRoles.length === 0) {
      return 'Career alignment cannot start until target roles and required skills are filled in.'
    }

    if (derived.gapSkills.length === 0) {
      return `Current input does not show open gaps for ${setup.targetRoles.join(', ')}. Keep building visible proof of work and journaling execution honestly.`
    }

    return `For ${setup.targetRoles.join(', ')}, the biggest open gap is ${derived.gapSkills[0]}. ${derived.careerActions[0] ?? 'Add a focused skill block today.'}`
  }

  if (derived.consistencyScore === null) {
    return 'Health and behavior tracking need more journal data before patterns are measurable.'
  }

  const rate =
    derived.completionRate === null ? 'No completion data yet.' : `Completion rate is ${Math.round(derived.completionRate * 100)}%.`
  return `Consistency score is ${derived.consistencyScore}/100. ${rate} ${derived.patternWarnings[0] ?? 'No major behavior drift detected yet.'}`
}
