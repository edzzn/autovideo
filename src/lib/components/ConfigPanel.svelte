<script lang="ts">
  import { Label } from "@/components/ui/label";
  import { Switch } from "@/components/ui/switch";
  import { Slider } from "@/components/ui/slider";
  import { Select, SelectContent, SelectItem, SelectTrigger } from "@/components/ui/select";
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
  import { pipelineStore } from "@/lib/stores/pipeline";

  let { config } = $derived($pipelineStore);

  const languages = [
    { value: "null", label: "Auto-detect" },
    { value: "en", label: "English" },
    { value: "es", label: "Spanish" },
    { value: "fr", label: "French" },
    { value: "de", label: "German" },
    { value: "it", label: "Italian" },
    { value: "pt", label: "Portuguese" },
    { value: "ja", label: "Japanese" },
    { value: "zh", label: "Chinese" }
  ];

  let silenceThreshold = $state([config.silence_threshold_db]);
  let silenceMinDuration = $state([config.silence_min_duration]);
  let cutMargin = $state([config.cut_margin]);
  let selectedLanguage = $state([config.language || "null"]);
</script>

<Card>
  <CardHeader>
    <CardTitle>Processing Options</CardTitle>
    <CardDescription>Configure how your video will be processed</CardDescription>
  </CardHeader>
  <CardContent class="space-y-6">
    <!-- Enhance Audio -->
    <div class="flex items-center justify-between">
      <div class="space-y-0.5">
        <Label>Enhance Audio</Label>
        <p class="text-sm text-muted-foreground">Apply noise reduction and loudness normalization</p>
      </div>
      <Switch
        checked={config.enhance_audio}
        onCheckedChange={(checked) => pipelineStore.updateConfig({ enhance_audio: checked })}
      />
    </div>

    <!-- Cut Silences -->
    <div class="flex items-center justify-between">
      <div class="space-y-0.5">
        <Label>Cut Silences</Label>
        <p class="text-sm text-muted-foreground">Automatically remove silent portions</p>
      </div>
      <Switch
        checked={config.cut_silences}
        onCheckedChange={(checked) => pipelineStore.updateConfig({ cut_silences: checked })}
      />
    </div>

    {#if config.cut_silences}
      <!-- Silence Threshold -->
      <div class="space-y-2">
        <div class="flex justify-between">
          <Label>Silence Threshold</Label>
          <span class="text-sm text-muted-foreground">{silenceThreshold[0]} dB</span>
        </div>
        <Slider
          bind:value={silenceThreshold}
          onValueChange={(value: number[]) => {
            silenceThreshold = value;
            pipelineStore.updateConfig({ silence_threshold_db: value[0] });
          }}
          min={-50}
          max={-10}
          step={1}
          type="multiple"
        />
      </div>

      <!-- Minimum Silence Duration -->
      <div class="space-y-2">
        <div class="flex justify-between">
          <Label>Minimum Silence Duration</Label>
          <span class="text-sm text-muted-foreground">{silenceMinDuration[0].toFixed(1)}s</span>
        </div>
        <Slider
          bind:value={silenceMinDuration}
          onValueChange={(value: number[]) => {
            silenceMinDuration = value;
            pipelineStore.updateConfig({ silence_min_duration: value[0] });
          }}
          min={0.1}
          max={2.0}
          step={0.1}
          type="multiple"
        />
      </div>

      <!-- Cut Margin -->
      <div class="space-y-2">
        <div class="flex justify-between">
          <Label>Cut Margin</Label>
          <span class="text-sm text-muted-foreground">{cutMargin[0].toFixed(1)}s</span>
        </div>
        <Slider
          bind:value={cutMargin}
          onValueChange={(value: number[]) => {
            cutMargin = value;
            pipelineStore.updateConfig({ cut_margin: value[0] });
          }}
          min={0}
          max={1.0}
          step={0.1}
          type="multiple"
        />
        <p class="text-xs text-muted-foreground">Preserve this much time around speech</p>
      </div>
    {/if}

    <!-- Language -->
    <div class="space-y-2">
      <Label>Transcription Language</Label>
      <Select
        bind:value={selectedLanguage}
        onValueChange={(value: string[]) => {
          if (value && value.length > 0) {
            selectedLanguage = value;
            pipelineStore.updateConfig({ language: value[0] === "null" ? null : value[0] });
          }
        }}
        type="multiple"
      >
        <SelectTrigger>
          {languages.find(l => l.value === selectedLanguage[0])?.label || "Select language"}
        </SelectTrigger>
        <SelectContent>
          {#each languages as lang}
            <SelectItem value={lang.value}>{lang.label}</SelectItem>
          {/each}
        </SelectContent>
      </Select>
    </div>
  </CardContent>
</Card>
