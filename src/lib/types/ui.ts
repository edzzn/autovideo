export type Screen = 'home' | 'processing' | 'editor' | 'done';

export interface StepperStage {
  id: string;
  label: string;
  status: 'pending' | 'active' | 'completed' | 'failed';
  progress?: number;
}
