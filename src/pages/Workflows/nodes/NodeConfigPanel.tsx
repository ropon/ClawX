/**
 * Node Configuration Panel
 * Right-side panel for configuring selected workflow node properties
 */
import { useEffect } from 'react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Plus, Trash2 } from 'lucide-react';
import { useAgentStore } from '@/stores/agents';
import { useTranslation } from 'react-i18next';
import type { WorkflowNode, WorkflowNodeData, ConditionRule } from '@/types/workflow';

interface NodeConfigPanelProps {
  node: WorkflowNode | null;
  onUpdate: (nodeId: string, data: Partial<WorkflowNodeData>) => void;
}

export function NodeConfigPanel({ node, onUpdate }: NodeConfigPanelProps) {
  const { t } = useTranslation('workflows');
  const agents = useAgentStore((s) => s.agents);
  const fetchAgents = useAgentStore((s) => s.fetchAgents);

  useEffect(() => {
    if (agents.length === 0) fetchAgents();
  }, [agents.length, fetchAgents]);

  if (!node) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground p-4">
        {t('editor.selectNode')}
      </div>
    );
  }

  const { data, type } = node;

  return (
    <div className="space-y-4 p-4 overflow-auto">
      {/* Label */}
      <div className="space-y-1.5">
        <Label>{t('name')}</Label>
        <Input
          value={data.label || ''}
          onChange={(e) => onUpdate(node.id, { label: e.target.value })}
          placeholder={t('namePlaceholder')}
        />
      </div>

      {/* Agent-specific config */}
      {type === 'agent' && (
        <>
          <div className="space-y-1.5">
            <Label>{t('editor.agentSelect')}</Label>
            <select
              className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm"
              value={data.agentId || ''}
              onChange={(e) => onUpdate(node.id, { agentId: e.target.value || undefined })}
            >
              <option value="">{t('editor.agentSelect')}</option>
              {agents.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.avatar} {a.name}
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-1.5">
            <Label>{t('editor.promptTemplate')}</Label>
            <Textarea
              rows={4}
              value={data.promptTemplate || ''}
              onChange={(e) => onUpdate(node.id, { promptTemplate: e.target.value })}
              placeholder={t('editor.promptTemplatePlaceholder')}
            />
          </div>
        </>
      )}

      {/* Condition-specific config */}
      {type === 'condition' && <ConditionConfig data={data} nodeId={node.id} onUpdate={onUpdate} />}

      {/* Merge-specific config */}
      {type === 'merge' && (
        <>
          <div className="space-y-1.5">
            <Label>{t('editor.mergeStrategy')}</Label>
            <select
              className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm"
              value={data.mergeStrategy || 'concat'}
              onChange={(e) =>
                onUpdate(node.id, {
                  mergeStrategy: e.target.value as 'concat' | 'first' | 'custom',
                })
              }
            >
              <option value="concat">{t('editor.mergeConcat')}</option>
              <option value="first">{t('editor.mergeFirst')}</option>
              <option value="custom">{t('editor.mergeCustom')}</option>
            </select>
          </div>
          {data.mergeStrategy === 'custom' && (
            <div className="space-y-1.5">
              <Label>{t('editor.mergeTemplate')}</Label>
              <Textarea
                rows={3}
                value={data.mergeTemplate || ''}
                onChange={(e) => onUpdate(node.id, { mergeTemplate: e.target.value })}
                placeholder={t('editor.mergeTemplatePlaceholder')}
              />
            </div>
          )}
        </>
      )}
    </div>
  );
}

function ConditionConfig({
  data,
  nodeId,
  onUpdate,
}: {
  data: WorkflowNodeData;
  nodeId: string;
  onUpdate: (nodeId: string, data: Partial<WorkflowNodeData>) => void;
}) {
  const { t } = useTranslation('workflows');
  const rules = data.conditionRules || [];

  const addRule = () => {
    const newRule: ConditionRule = {
      id: `rule-${Date.now()}`,
      handle: `branch_${rules.length}`,
      type: data.conditionType || 'keyword',
      value: '',
    };
    onUpdate(nodeId, { conditionRules: [...rules, newRule] });
  };

  const updateRule = (index: number, updates: Partial<ConditionRule>) => {
    const updated = rules.map((r, i) => (i === index ? { ...r, ...updates } : r));
    onUpdate(nodeId, { conditionRules: updated });
  };

  const removeRule = (index: number) => {
    onUpdate(nodeId, { conditionRules: rules.filter((_, i) => i !== index) });
  };

  return (
    <>
      <div className="space-y-1.5">
        <Label>{t('editor.conditionType')}</Label>
        <select
          className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm"
          value={data.conditionType || 'keyword'}
          onChange={(e) =>
            onUpdate(nodeId, {
              conditionType: e.target.value as 'keyword' | 'regex' | 'ai-classify',
            })
          }
        >
          <option value="keyword">Keyword</option>
          <option value="regex">Regex</option>
          <option value="ai-classify">AI Classify</option>
        </select>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label>{t('editor.conditionRules')}</Label>
          <Button variant="ghost" size="sm" onClick={addRule}>
            <Plus className="h-3 w-3 mr-1" />
            {t('editor.addRule')}
          </Button>
        </div>
        {rules.map((rule, i) => (
          <div key={rule.id} className="flex gap-2 items-center">
            <Input
              className="flex-1"
              placeholder={t('editor.ruleValue')}
              value={rule.value}
              onChange={(e) => updateRule(i, { value: e.target.value })}
            />
            <Input
              className="w-24"
              placeholder={t('editor.ruleHandle')}
              value={rule.handle}
              onChange={(e) => updateRule(i, { handle: e.target.value })}
            />
            <Button variant="ghost" size="icon" onClick={() => removeRule(i)}>
              <Trash2 className="h-3.5 w-3.5 text-destructive" />
            </Button>
          </div>
        ))}
      </div>
    </>
  );
}
