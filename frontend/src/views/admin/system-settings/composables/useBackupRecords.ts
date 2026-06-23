import { computed, ref } from 'vue'
import { adminApi, type BackupRunRecord, type BackupRunStatus } from '@/api/admin'
import { log } from '@/utils/logger'

type BadgeVariant = 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning' | 'dark'

const statusText = {
  queued: '执行中',
  running: '执行中',
  retrying: '执行中',
  succeeded: '成功',
  failed: '失败',
  cancelled: '已取消',
  skipped: '已跳过',
} satisfies Record<BackupRunStatus, string>

const statusBadgeVariant = {
  queued: 'warning',
  running: 'warning',
  retrying: 'warning',
  succeeded: 'success',
  failed: 'destructive',
  cancelled: 'secondary',
  skipped: 'secondary',
} satisfies Record<BackupRunStatus, BadgeVariant>

export function formatBackupDateTime(value: string | null | undefined): string {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return '-'
  return date.toLocaleString('zh-CN')
}

export function backupRunStatusText(status: BackupRunStatus): string {
  return statusText[status]
}

export function backupRunStatusBadgeVariant(status: BackupRunStatus): BadgeVariant {
  return statusBadgeVariant[status]
}

export function backupRunTimeText(run: BackupRunRecord): string {
  return formatBackupDateTime(run.finished_at ?? run.started_at ?? run.created_at)
}

export function backupRunDetailText(run: BackupRunRecord): string {
  if (run.error_message) return run.error_message
  if (run.progress_message) return run.progress_message
  if (run.created_by) return `操作者：${run.created_by}`
  return run.trigger === 'manual' ? '手动触发' : run.trigger
}

export function formatBackupRunProgress(run: BackupRunRecord): string {
  const value = Math.max(0, Math.min(100, Math.round(run.progress_percent)))
  return `${value}%`
}

export function useBackupRecords() {
  const backupRuns = ref<readonly BackupRunRecord[]>([])
  const totalBackupRuns = ref(0)
  const backupRecordsLoading = ref(false)
  const backupRecordsError = ref<string | null>(null)

  const latestBackupRun = computed(() => backupRuns.value[0] ?? null)
  const latestBackupStatusText = computed(() => {
    return latestBackupRun.value ? backupRunStatusText(latestBackupRun.value.status) : '暂无记录'
  })
  const latestBackupTimeText = computed(() => {
    return latestBackupRun.value ? backupRunTimeText(latestBackupRun.value) : '-'
  })

  async function loadBackupRecords() {
    backupRecordsLoading.value = true
    backupRecordsError.value = null
    try {
      const response = await adminApi.getBackupRuns()
      backupRuns.value = response.items.slice(0, 10)
      totalBackupRuns.value = response.total
    } catch (error) {
      backupRecordsError.value = '备份记录加载失败'
      log.error('备份记录加载失败:', error)
    } finally {
      backupRecordsLoading.value = false
    }
  }

  return {
    backupRuns,
    totalBackupRuns,
    backupRecordsLoading,
    backupRecordsError,
    latestBackupRun,
    latestBackupStatusText,
    latestBackupTimeText,
    loadBackupRecords,
    formatBackupRunTime: backupRunTimeText,
    backupRunDetailText,
    backupRunStatusText,
    backupRunStatusBadgeVariant,
    formatBackupRunProgress,
  }
}
