import { RiskLevel } from './store';

export function toneForRisk(risk: RiskLevel) {
  if (risk === RiskLevel.Critical || risk === RiskLevel.High) return 'danger';
  if (risk === RiskLevel.Medium) return 'warn';
  if (risk === RiskLevel.Low) return 'ok';
  return 'info';
}
