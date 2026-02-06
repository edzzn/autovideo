import { writable, get } from 'svelte/store';
import type { PipelineConfig, PipelineEvent, PipelineResult } from '$lib/types/pipeline';
import type { Screen, StepperStage } from '$lib/types/ui';

interface PipelineStore {
  screen: Screen;
  selectedFile: string | null;
  config: PipelineConfig;
  stages: StepperStage[];
  result: PipelineResult | null;
  error: string | null;
  isProcessing: boolean;
}

const defaultConfig: PipelineConfig = {
  enhance_audio: true,
  cut_silences: true,
  silence_threshold_db: -30.0,
  silence_min_duration: 0.5,
  cut_margin: 0.2,
  language: null
};

const initialStages: StepperStage[] = [
  { id: 'transcribe', label: 'Transcribing Audio', status: 'pending' },
  { id: 'detect_silences', label: 'Detecting Silences', status: 'pending' },
  { id: 'cut_silences', label: 'Cutting Silences', status: 'pending' },
  { id: 'enhance_audio', label: 'Enhancing Audio', status: 'pending' },
  { id: 'export', label: 'Exporting Video', status: 'pending' }
];

const initialState: PipelineStore = {
  screen: 'home',
  selectedFile: null,
  config: { ...defaultConfig },
  stages: initialStages.map(s => ({ ...s })),
  result: null,
  error: null,
  isProcessing: false
};

function createPipelineStore() {
  const store = writable<PipelineStore>(initialState);
  const { subscribe, set, update } = store;

  return {
    subscribe,

    get: () => get(store),

    setScreen: (screen: Screen) => {
      update(state => ({ ...state, screen, error: null }));
    },

    setFile: (filePath: string | null) => {
      update(state => ({ ...state, selectedFile: filePath, error: null }));
    },

    updateConfig: (config: Partial<PipelineConfig>) => {
      update(state => ({
        ...state,
        config: { ...state.config, ...config }
      }));
    },

    startProcessing: () => {
      update(state => ({
        ...state,
        screen: 'processing',
        isProcessing: true,
        error: null,
        stages: initialStages.map(s => ({ ...s })),
        result: null
      }));
    },

    handleEvent: (event: PipelineEvent) => {
      console.log('📊 Store handling event:', event);

      update(state => {
        const newState = { ...state };

        if ('StageStarted' in event) {
          const { stage } = event.StageStarted;
          console.log(`▶️ Stage started: ${stage}`);
          newState.stages = state.stages.map(s =>
            s.id === stage ? { ...s, status: 'active' as const, progress: 0 } : s
          );
        } else if ('StageProgress' in event) {
          const { stage, progress } = event.StageProgress;
          console.log(`⏳ Stage progress: ${stage} - ${(progress * 100).toFixed(1)}%`);
          newState.stages = state.stages.map(s =>
            s.id === stage ? { ...s, status: 'active' as const, progress } : s
          );
        } else if ('StageCompleted' in event) {
          const { stage } = event.StageCompleted;
          console.log(`✅ Stage completed: ${stage}`);
          newState.stages = state.stages.map(s =>
            s.id === stage ? { ...s, status: 'completed' as const, progress: 1 } : s
          );
        } else if ('StageFailed' in event) {
          const { stage, error } = event.StageFailed;
          console.error(`❌ Stage failed: ${stage}`, error);
          newState.stages = state.stages.map(s =>
            s.id === stage ? { ...s, status: 'failed' as const } : s
          );
          newState.error = `Stage "${stage}" failed: ${error}`;
          newState.isProcessing = false;
        } else if ('PipelineCompleted' in event) {
          const { result } = event.PipelineCompleted;
          console.log('🎉 Pipeline completed!', result);
          newState.result = result;
          newState.screen = 'done';
          newState.isProcessing = false;
        } else if ('PipelineFailed' in event) {
          const { error } = event.PipelineFailed;
          console.error('❌ Pipeline failed:', error);
          newState.error = `Pipeline failed: ${error}`;
          newState.screen = 'home';
          newState.isProcessing = false;
        }

        return newState;
      });
    },

    reset: () => {
      set({
        ...initialState,
        config: { ...defaultConfig },
        stages: initialStages.map(s => ({ ...s }))
      });
    }
  };
}

export const pipelineStore = createPipelineStore();
