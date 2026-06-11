import type { ElementOf } from 'duoyun-ui/lib/types';

import { convertToMap } from 'duoyun-ui/lib/utils';

import { ApiType, AuditStatus, LogLevel, RiskLevel } from './store';

export const apiTypeList: { value: ApiType; label: string }[] = [
  { value: ApiType.OpenAI, label: 'OpenAI' },
  { value: ApiType.Anthropic, label: 'Anthropic' },
];

export const apiTypeMap = convertToMap<ElementOf<typeof apiTypeList>, string>(apiTypeList, 'value', 'label');

export const logLevelList: { value: LogLevel; label: string }[] = [
  { value: LogLevel.Info, label: 'Info' },
  { value: LogLevel.Warn, label: 'Warn' },
  { value: LogLevel.Error, label: 'Error' },
];

export const logLevelMap = convertToMap<ElementOf<typeof logLevelList>, string>(logLevelList, 'value', 'label');

export const riskLevelList: { value: RiskLevel; label: string }[] = [
  { value: RiskLevel.Unknown, label: 'Unknown' },
  { value: RiskLevel.Low, label: 'Low' },
  { value: RiskLevel.Medium, label: 'Medium' },
  { value: RiskLevel.High, label: 'High' },
  { value: RiskLevel.Critical, label: 'Critical' },
];

export const riskLevelMap = convertToMap<ElementOf<typeof riskLevelList>, string>(riskLevelList, 'value', 'label');

export const auditStatusList: { value: AuditStatus; label: string }[] = [
  { value: AuditStatus.Queued, label: 'Queued' },
  { value: AuditStatus.Completed, label: 'Completed' },
  { value: AuditStatus.Failed, label: 'Failed' },
  { value: AuditStatus.Skipped, label: 'Skipped' },
];

export const auditStatusMap = convertToMap<ElementOf<typeof auditStatusList>, string>(
  auditStatusList,
  'value',
  'label',
);
