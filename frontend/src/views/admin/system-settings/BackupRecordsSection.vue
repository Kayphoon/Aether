<template>
  <CardSection
    title="备份记录"
    description="查看完整备份导出的最近执行情况"
  >
    <template #actions>
      <Button
        variant="outline"
        size="sm"
        :disabled="loading"
        @click="$emit('refresh')"
      >
        <RefreshCw
          class="w-3.5 h-3.5 mr-1.5"
          :class="{ 'animate-spin': loading }"
        />
        刷新
      </Button>
    </template>

    <div class="space-y-4">
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
        <div class="rounded-lg border border-border bg-muted/30 px-4 py-3">
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <Activity class="w-3.5 h-3.5" />
            最新状态
          </div>
          <div class="mt-2">
            <Badge :variant="latestRun ? backupRunStatusBadgeVariant(latestRun.status) : 'secondary'">
              {{ latestStatusText }}
            </Badge>
          </div>
        </div>

        <div class="rounded-lg border border-border bg-muted/30 px-4 py-3">
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <Archive class="w-3.5 h-3.5" />
            备份总数
          </div>
          <div class="mt-2 text-lg font-semibold text-foreground">
            {{ total }}
          </div>
        </div>

        <div class="rounded-lg border border-border bg-muted/30 px-4 py-3">
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <Clock3 class="w-3.5 h-3.5" />
            最近时间
          </div>
          <div class="mt-2 truncate text-sm font-medium text-foreground">
            {{ latestTimeText }}
          </div>
        </div>
      </div>

      <div
        v-if="error"
        class="rounded-lg border border-border bg-muted/30 px-4 py-3 text-sm"
      >
        <div class="font-medium text-destructive">
          {{ error }}
        </div>
        <p class="mt-1 text-xs text-muted-foreground">
          请稍后重试，或检查后台任务服务是否正常。
        </p>
      </div>

      <div class="rounded-lg border border-border overflow-hidden">
        <div class="flex items-center justify-between px-4 py-3 border-b border-border">
          <div>
            <h4 class="text-sm font-medium">
              最近备份
            </h4>
            <p class="text-xs text-muted-foreground">
              完整备份导出任务的最近执行结果
            </p>
          </div>
        </div>

        <div
          v-if="loading && runs.length === 0"
          class="px-4 py-6 text-sm text-muted-foreground"
        >
          备份记录加载中...
        </div>

        <div
          v-else-if="runs.length === 0"
          class="px-4 py-6 text-sm text-muted-foreground"
        >
          暂无备份记录
        </div>

        <div
          v-else
          class="overflow-x-auto"
        >
          <table class="w-full text-sm">
            <thead class="bg-muted/30 text-xs text-muted-foreground">
              <tr>
                <th class="px-4 py-2 text-left font-medium">
                  时间
                </th>
                <th class="px-4 py-2 text-left font-medium">
                  状态
                </th>
                <th class="px-4 py-2 text-left font-medium">
                  进度
                </th>
                <th class="px-4 py-2 text-left font-medium">
                  触发方式
                </th>
                <th class="px-4 py-2 text-left font-medium">
                  详情
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="run in runs"
                :key="run.id"
                class="border-t border-border"
              >
                <td class="px-4 py-2 whitespace-nowrap">
                  {{ backupRunTimeText(run) }}
                </td>
                <td class="px-4 py-2 whitespace-nowrap">
                  <Badge :variant="backupRunStatusBadgeVariant(run.status)">
                    {{ backupRunStatusText(run.status) }}
                  </Badge>
                </td>
                <td class="px-4 py-2 whitespace-nowrap text-muted-foreground">
                  {{ formatBackupRunProgress(run) }}
                </td>
                <td class="px-4 py-2 whitespace-nowrap text-muted-foreground">
                  {{ triggerText(run.trigger) }}
                </td>
                <td class="px-4 py-2 min-w-[18rem]">
                  <div class="line-clamp-2 text-muted-foreground">
                    {{ backupRunDetailText(run) }}
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { Activity, Archive, Clock3, RefreshCw } from 'lucide-vue-next'
import type { BackupRunRecord } from '@/api/admin'
import { Badge, Button } from '@/components/ui'
import { CardSection } from '@/components/layout'
import {
  backupRunDetailText,
  backupRunStatusBadgeVariant,
  backupRunStatusText,
  backupRunTimeText,
  formatBackupRunProgress,
} from './composables/useBackupRecords'

defineProps<{
  runs: readonly BackupRunRecord[]
  total: number
  latestRun: BackupRunRecord | null
  latestStatusText: string
  latestTimeText: string
  loading: boolean
  error: string | null
}>()

defineEmits<{
  refresh: []
}>()

function triggerText(trigger: string): string {
  return trigger === 'manual' ? '手动' : trigger
}
</script>
