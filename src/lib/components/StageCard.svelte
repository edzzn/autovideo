<script lang="ts">
  import { Card, CardContent } from "@/components/ui/card";
  import { Progress } from "@/components/ui/progress";
  import { Badge } from "@/components/ui/badge";
  import { Circle, Loader2, CheckCircle, AlertCircle } from "lucide-svelte";
  import type { StepperStage } from "@/lib/types/ui";

  interface Props {
    stage: StepperStage;
  }

  let { stage }: Props = $props();

  const statusConfig = {
    pending: {
      icon: Circle,
      color: "text-muted-foreground",
      badgeVariant: "secondary" as const,
      label: "Pending"
    },
    active: {
      icon: Loader2,
      color: "text-primary",
      badgeVariant: "default" as const,
      label: "In Progress"
    },
    completed: {
      icon: CheckCircle,
      color: "text-green-500",
      badgeVariant: "default" as const,
      label: "Completed"
    },
    failed: {
      icon: AlertCircle,
      color: "text-destructive",
      badgeVariant: "destructive" as const,
      label: "Failed"
    }
  };

  let config = $derived(statusConfig[stage.status]);
  let Icon = $derived(config.icon);
  let showProgress = $derived(stage.status === 'active' && stage.progress !== undefined);
</script>

<Card>
  <CardContent class="p-6">
    <div class="flex items-start gap-4">
      <!-- Icon -->
      <div class={`flex-shrink-0 ${config.color}`}>
        {#if stage.status === 'active'}
          <Icon class="h-6 w-6 animate-spin" />
        {:else}
          <Icon class="h-6 w-6" />
        {/if}
      </div>

      <!-- Content -->
      <div class="flex-1 space-y-2">
        <div class="flex items-center justify-between">
          <h3 class="font-semibold">{stage.label}</h3>
          <Badge variant={config.badgeVariant}>{config.label}</Badge>
        </div>

        {#if showProgress}
          <div class="space-y-1">
            <Progress value={(stage.progress || 0) * 100} />
            <p class="text-xs text-muted-foreground text-right">
              {Math.round((stage.progress || 0) * 100)}%
            </p>
          </div>
        {/if}
      </div>
    </div>
  </CardContent>
</Card>
