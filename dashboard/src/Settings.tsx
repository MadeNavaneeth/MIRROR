import { Save, X } from 'lucide-react'
import type { MirrorSetup } from './mirror'
import { formatLineList, parseLineList } from './mirror'

function Field({
  label,
  hint,
  children,
}: {
  label: string
  hint?: string
  children: React.ReactNode
}) {
  return (
    <label className="setup-field">
      <span className="setup-label">{label}</span>
      {children}
      {hint ? <span className="setup-hint">{hint}</span> : null}
    </label>
  )
}

function Section({
  title,
  description,
  children,
}: {
  title: string
  description: string
  children: React.ReactNode
}) {
  return (
    <section className="setup-section">
      <div className="setup-section-header">
        <div className="setup-eyebrow">{title}</div>
        <h3 className="setup-title">{description}</h3>
      </div>
      <div className="setup-section-body">{children}</div>
    </section>
  )
}

export function Settings({
  setup,
  onChange,
  onClose,
}: {
  setup: MirrorSetup
  onChange: (next: MirrorSetup) => void
  onClose: () => void
}) {
  const update = <K extends keyof MirrorSetup>(key: K, value: MirrorSetup[K]) => {
    onChange({
      ...setup,
      [key]: value,
    })
  }

  return (
    <div className="setup-overlay">
      <div className="setup-panel">
        <header className="setup-header">
          <div>
            <div className="setup-eyebrow">System setup</div>
            <h2 className="setup-heading">Mirror initialization + operating baseline</h2>
            <p className="setup-copy">
              This panel collects the minimum data required for the four-agent system to plan,
              journal, align career work, and measure behavior honestly.
            </p>
          </div>
          <button type="button" className="icon-button" onClick={onClose}>
            <X size={16} />
          </button>
        </header>

        <div className="setup-scroll">
          <Section
            title="Career Agent"
            description="Target roles, current assets, missing-skill expectations, and proof-of-work context."
          >
            <div className="setup-grid two-up">
              <Field label="Target roles" hint="One role per line. Example: SDE, Game Developer, ML Engineer.">
                <textarea
                  className="control control-textarea"
                  rows={4}
                  value={formatLineList(setup.targetRoles)}
                  onChange={(event) => update('targetRoles', parseLineList(event.target.value))}
                />
              </Field>
              <Field label="Current skills" hint="Your actual current skill inventory. One skill per line.">
                <textarea
                  className="control control-textarea"
                  rows={4}
                  value={formatLineList(setup.currentSkills)}
                  onChange={(event) => update('currentSkills', parseLineList(event.target.value))}
                />
              </Field>
              <Field label="Required skills" hint="Paste the skill expectations for your chosen roles. One skill per line.">
                <textarea
                  className="control control-textarea"
                  rows={5}
                  value={formatLineList(setup.requiredSkills)}
                  onChange={(event) => update('requiredSkills', parseLineList(event.target.value))}
                />
              </Field>
              <Field label="Weak areas" hint="Optional self-reported weak spots so the system can map them explicitly.">
                <textarea
                  className="control control-textarea"
                  rows={5}
                  value={formatLineList(setup.weakAreas)}
                  onChange={(event) => update('weakAreas', parseLineList(event.target.value))}
                />
              </Field>
              <Field label="Resume notes" hint="Paste summary bullets, resume text, or important context.">
                <textarea
                  className="control control-textarea"
                  rows={6}
                  value={setup.resumeNotes}
                  onChange={(event) => update('resumeNotes', event.target.value)}
                />
              </Field>
              <Field label="Portfolio / GitHub / supporting links" hint="One link per line.">
                <textarea
                  className="control control-textarea"
                  rows={6}
                  value={formatLineList(setup.portfolioLinks)}
                  onChange={(event) => update('portfolioLinks', parseLineList(event.target.value))}
                />
              </Field>
            </div>
          </Section>

          <Section
            title="Planner Agent"
            description="Daily structure, timetable anchors, and constraints the planner should respect."
          >
            <div className="setup-grid four-up">
              <Field label="Day start">
                <input
                  className="control"
                  type="time"
                  value={setup.dayStart}
                  onChange={(event) => update('dayStart', event.target.value)}
                />
              </Field>
              <Field label="Work / study start">
                <input
                  className="control"
                  type="time"
                  value={setup.workStart}
                  onChange={(event) => update('workStart', event.target.value)}
                />
              </Field>
              <Field label="Work / study end">
                <input
                  className="control"
                  type="time"
                  value={setup.workEnd}
                  onChange={(event) => update('workEnd', event.target.value)}
                />
              </Field>
              <Field label="Day end">
                <input
                  className="control"
                  type="time"
                  value={setup.dayEnd}
                  onChange={(event) => update('dayEnd', event.target.value)}
                />
              </Field>
              <Field label="Skill block (hours)" hint="Dedicated daily career-growth time.">
                <input
                  className="control"
                  type="number"
                  min="0"
                  step="0.5"
                  value={setup.skillBlockHours}
                  onChange={(event) => update('skillBlockHours', Number(event.target.value))}
                />
              </Field>
              <Field label="Health block (minutes)" hint="Movement, breaks, sleep prep, or recovery.">
                <input
                  className="control"
                  type="number"
                  min="0"
                  step="5"
                  value={setup.healthBlockMinutes}
                  onChange={(event) => update('healthBlockMinutes', Number(event.target.value))}
                />
              </Field>
              <Field label="Break size (minutes)" hint="The default buffer between schedule blocks.">
                <input
                  className="control"
                  type="number"
                  min="0"
                  step="5"
                  value={setup.breakMinutes}
                  onChange={(event) => update('breakMinutes', Number(event.target.value))}
                />
              </Field>
              <Field label="Office days" hint="One per line. Example: Mon, Tue, Wed.">
                <textarea
                  className="control control-textarea"
                  rows={4}
                  value={formatLineList(setup.officeDays)}
                  onChange={(event) => update('officeDays', parseLineList(event.target.value))}
                />
              </Field>
            </div>

            <div className="setup-grid two-up">
              <Field label="Calendar notes" hint="Paste known events, recurring classes, or context if calendar sync is unavailable.">
                <textarea
                  className="control control-textarea"
                  rows={5}
                  value={setup.calendarNotes}
                  onChange={(event) => update('calendarNotes', event.target.value)}
                />
              </Field>
              <Field label="Timetable notes" hint="Add commute, office timing, lectures, or anything the planner must remember.">
                <textarea
                  className="control control-textarea"
                  rows={5}
                  value={setup.timetableNotes}
                  onChange={(event) => update('timetableNotes', event.target.value)}
                />
              </Field>
            </div>
          </Section>

          <Section
            title="Health / Behavior Agent"
            description="Baseline targets used to measure consistency, drift, and recovery."
          >
            <div className="setup-grid two-up">
              <Field label="Sleep target (hours)" hint="Used when sleep data is logged in journal entries.">
                <input
                  className="control"
                  type="number"
                  min="0"
                  step="0.5"
                  value={setup.sleepTargetHours}
                  onChange={(event) => update('sleepTargetHours', Number(event.target.value))}
                />
              </Field>
              <Field label="Workout target / week" hint="Simple weekly movement target for the health agent.">
                <input
                  className="control"
                  type="number"
                  min="0"
                  step="1"
                  value={setup.workoutTargetPerWeek}
                  onChange={(event) => update('workoutTargetPerWeek', Number(event.target.value))}
                />
              </Field>
            </div>
          </Section>

          <Section
            title="Journal Agent"
            description="Supplemental context and documents the always-on input engine should keep available."
          >
            <div className="setup-grid two-up">
              <Field label="Supporting documents" hint="One path, URL, or note per line. Use this for extra material beyond resume and portfolio.">
                <textarea
                  className="control control-textarea"
                  rows={6}
                  value={formatLineList(setup.supportingDocuments)}
                  onChange={(event) => update('supportingDocuments', parseLineList(event.target.value))}
                />
              </Field>
              <Field label="What this system should understand about your life" hint="Free-form context: routines, obligations, current friction, or constraints.">
                <textarea
                  className="control control-textarea"
                  rows={6}
                  value={setup.calendarNotes + (setup.calendarNotes && setup.timetableNotes ? '\n\n' : '') + setup.timetableNotes}
                  onChange={(event) => {
                    const [calendarNotes = '', ...rest] = event.target.value.split('\n\n')
                    update('calendarNotes', calendarNotes)
                    update('timetableNotes', rest.join('\n\n'))
                  }}
                />
              </Field>
            </div>
          </Section>
        </div>

        <footer className="setup-footer">
          <div className="setup-note">
            Inputs are stored locally in the browser so the system can keep intention vs action visible across sessions.
          </div>
          <button type="button" className="primary-button" onClick={onClose}>
            <Save size={14} />
            Save and return
          </button>
        </footer>
      </div>
    </div>
  )
}
