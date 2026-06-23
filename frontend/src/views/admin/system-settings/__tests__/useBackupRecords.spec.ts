import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createApp, defineComponent, h, type App } from 'vue'

import type { BackupRunListResponse } from '@/api/admin'

const { getBackupRunsMock, logErrorMock } = vi.hoisted(() => ({
  getBackupRunsMock: vi.fn<() => Promise<BackupRunListResponse>>(),
  logErrorMock: vi.fn(),
}))

vi.mock('@/api/admin', () => ({
  adminApi: {
    getBackupRuns: getBackupRunsMock,
  },
}))

vi.mock('@/utils/logger', () => ({
  log: {
    error: logErrorMock,
  },
}))

vi.mock('@/components/layout', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    CardSection: defineComponent({
      name: 'CardSectionStub',
      props: {
        title: String,
        description: String,
      },
      setup(props, { slots }) {
        return () => h('section', [
          h('h2', props.title),
          h('p', props.description),
          slots.actions?.(),
          slots.default?.(),
        ])
      },
    }),
  }
})

vi.mock('@/components/ui', async () => {
  const { defineComponent, h } = await import('vue')
  return {
    Badge: defineComponent({
      name: 'BadgeStub',
      props: {
        variant: String,
      },
      setup(props, { slots }) {
        return () => h('span', { 'data-variant': props.variant }, slots.default?.())
      },
    }),
    Button: defineComponent({
      name: 'ButtonStub',
      props: {
        disabled: Boolean,
      },
      setup(props, { slots, emit }) {
        return () => h('button', {
          disabled: props.disabled,
          onClick: () => emit('click'),
        }, slots.default?.())
      },
    }),
  }
})

import { useBackupRecords } from '../composables/useBackupRecords'
import BackupRecordsSection from '../BackupRecordsSection.vue'

const completedRun = {
  id: 'task-1',
  task_key: 'admin.system.data_export',
  kind: 'on_demand',
  trigger: 'manual',
  status: 'succeeded',
  attempt: 1,
  max_attempts: 1,
  owner_instance: 'gateway-1',
  progress_percent: 100,
  progress_message: '导出完成',
  payload: { scope: 'complete_backup' },
  result: { exported_bytes: 512 },
  error_message: null,
  cancel_requested: false,
  created_by: 'admin@example.com',
  created_at: '2026-06-22T08:30:00Z',
  started_at: '2026-06-22T08:30:03Z',
  finished_at: '2026-06-22T08:30:12Z',
  updated_at: '2026-06-22T08:30:12Z',
} satisfies BackupRunListResponse['items'][number]

const runningRun = {
  ...completedRun,
  id: 'task-2',
  status: 'running',
  progress_percent: 45,
  progress_message: '正在写入备份',
  created_at: '2026-06-23T10:00:00Z',
  started_at: '2026-06-23T10:00:02Z',
  finished_at: null,
  updated_at: '2026-06-23T10:00:10Z',
} satisfies BackupRunListResponse['items'][number]

const mountedApps: Array<{ app: App, root: HTMLElement }> = []

function mountBackupRecordsSection(props: InstanceType<typeof BackupRecordsSection>['$props']) {
  const root = document.createElement('div')
  document.body.appendChild(root)
  const app = createApp(BackupRecordsSection, props)
  app.mount(root)
  mountedApps.push({ app, root })
  return root
}

describe('useBackupRecords', () => {
  beforeEach(() => {
    getBackupRunsMock.mockReset()
    logErrorMock.mockReset()
  })

  afterEach(() => {
    for (const { app, root } of mountedApps.splice(0)) {
      app.unmount()
      root.remove()
    }
    document.body.innerHTML = ''
  })

  it('loads recent backup records and derives latest summary', async () => {
    getBackupRunsMock.mockResolvedValue({
      items: [runningRun, completedRun],
      total: 2,
      page: 1,
      page_size: 10,
      pages: 1,
    })

    const records = useBackupRecords()

    await records.loadBackupRecords()

    expect(getBackupRunsMock).toHaveBeenCalledTimes(1)
    expect(records.backupRuns.value).toEqual([runningRun, completedRun])
    expect(records.totalBackupRuns.value).toBe(2)
    expect(records.latestBackupRun.value).toEqual(runningRun)
    expect(records.latestBackupStatusText.value).toBe('执行中')
    expect(records.latestBackupTimeText.value).toContain('2026')
    expect(records.backupRecordsError.value).toBeNull()
  })

  it('keeps an empty state when the backend has no backup records', async () => {
    getBackupRunsMock.mockResolvedValue({
      items: [],
      total: 0,
      page: 1,
      page_size: 10,
      pages: 0,
    })

    const records = useBackupRecords()

    await records.loadBackupRecords()

    expect(records.backupRuns.value).toEqual([])
    expect(records.totalBackupRuns.value).toBe(0)
    expect(records.latestBackupRun.value).toBeNull()
    expect(records.latestBackupStatusText.value).toBe('暂无记录')
    expect(records.latestBackupTimeText.value).toBe('-')
  })

  it('surfaces a readable error when loading backup records fails', async () => {
    getBackupRunsMock.mockRejectedValue(new Error('network down'))

    const records = useBackupRecords()

    await records.loadBackupRecords()

    expect(records.backupRuns.value).toEqual([])
    expect(records.backupRecordsError.value).toBe('备份记录加载失败')
    expect(records.backupRecordsLoading.value).toBe(false)
    expect(logErrorMock).toHaveBeenCalledOnce()
  })

  it('renders backup record summary and recent rows', () => {
    const root = mountBackupRecordsSection({
      runs: [completedRun],
      total: 1,
      latestRun: completedRun,
      latestStatusText: '成功',
      latestTimeText: '2026/6/22 16:30:12',
      loading: false,
      error: null,
    })

    expect(root.textContent).toContain('备份记录')
    expect(root.textContent).toContain('最新状态')
    expect(root.textContent).toContain('备份总数')
    expect(root.textContent).toContain('最近备份')
    expect(root.textContent).toContain('成功')
    expect(root.textContent).toContain('100%')
    expect(root.textContent).toContain('手动')
    expect(root.textContent).toContain('导出完成')
  })

  it('renders backup record empty and error states', () => {
    const root = mountBackupRecordsSection({
      runs: [],
      total: 0,
      latestRun: null,
      latestStatusText: '暂无记录',
      latestTimeText: '-',
      loading: false,
      error: '备份记录加载失败',
    })

    expect(root.textContent).toContain('暂无记录')
    expect(root.textContent).toContain('暂无备份记录')
    expect(root.textContent).toContain('备份记录加载失败')
  })
})
