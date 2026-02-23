/**
 * Agent Editor
 * Dialog for creating and editing agent configurations
 */
import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Select } from '@/components/ui/select';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Checkbox } from '@/components/ui/checkbox';
import { useAgentStore } from '@/stores/agents';
import { useProviderStore } from '@/stores/providers';
import { useChannelsStore } from '@/stores/channels';
import { useKnowledgeStore } from '@/stores/knowledge';
import { CHANNEL_ICONS, CHANNEL_NAMES, type ChannelType } from '@/types/channel';
import { toast } from 'sonner';
import { useTranslation } from 'react-i18next';
import type { AgentConfig } from '@/types/agent';
import type { AgentTemplate } from '@/data/agent-templates';

const EMOJI_OPTIONS = [
  '🤖', '🧠', '💡', '🎯', '🔍', '📝', '💻', '🎨',
  '📊', '🔧', '🚀', '🌐', '📚', '🎓', '🏗️', '⚡',
  '🔬', '🎭', '🛡️', '📐', '🧪', '🎵', '🌟', '👨‍💻',
];

interface AgentEditorProps {
  open: boolean;
  agent: AgentConfig | null;
  template?: AgentTemplate | null;
  onClose: () => void;
}

interface FormData {
  name: string;
  avatar: string;
  description: string;
  systemPrompt: string;
  providerId: string;
  model: string;
  temperature: number;
  maxTokens: string;
  channelBindings: string[];
  knowledgeBaseIds: string[];
}

export function AgentEditor({ open, agent, template, onClose }: AgentEditorProps) {
  const { t } = useTranslation('agents');
  const { t: tc } = useTranslation('common');
  const createAgent = useAgentStore((s) => s.createAgent);
  const updateAgent = useAgentStore((s) => s.updateAgent);
  const providers = useProviderStore((s) => s.providers);
  const fetchProviders = useProviderStore((s) => s.fetchProviders);
  const channels = useChannelsStore((s) => s.channels);
  const fetchChannels = useChannelsStore((s) => s.fetchChannels);
  const knowledgeBases = useKnowledgeStore((s) => s.knowledgeBases);
  const fetchKnowledgeBases = useKnowledgeStore((s) => s.fetchKnowledgeBases);

  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState<FormData>({
    name: '',
    avatar: '🤖',
    description: '',
    systemPrompt: '',
    providerId: '',
    model: '',
    temperature: 0.7,
    maxTokens: '',
    channelBindings: [],
    knowledgeBaseIds: [],
  });

  useEffect(() => {
    if (open) {
      fetchProviders();
      fetchChannels();
      fetchKnowledgeBases();
      if (agent) {
        setForm({
          name: agent.name,
          avatar: agent.avatar,
          description: agent.description,
          systemPrompt: agent.systemPrompt,
          providerId: agent.providerId || '',
          model: agent.model,
          temperature: agent.temperature,
          maxTokens: agent.maxTokens ? String(agent.maxTokens) : '',
          channelBindings: agent.channelBindings || [],
          knowledgeBaseIds: agent.knowledgeBaseIds || [],
        });
      } else if (template) {
        setForm({
          name: t(template.nameKey),
          avatar: template.avatar,
          description: t(template.descriptionKey),
          systemPrompt: template.systemPrompt,
          providerId: '',
          model: '',
          temperature: template.temperature,
          maxTokens: '',
          channelBindings: [],
          knowledgeBaseIds: [],
        });
      } else {
        setForm({
          name: '',
          avatar: '🤖',
          description: '',
          systemPrompt: '',
          providerId: '',
          model: '',
          temperature: 0.7,
          maxTokens: '',
          channelBindings: [],
          knowledgeBaseIds: [],
        });
      }
    }
  }, [open, agent, template, fetchProviders, fetchChannels, fetchKnowledgeBases, t]);

  const handleSave = async () => {
    if (!form.name.trim()) {
      toast.error('Name is required');
      return;
    }

    setSaving(true);
    try {
      const config = {
        name: form.name.trim(),
        avatar: form.avatar,
        description: form.description.trim(),
        systemPrompt: form.systemPrompt,
        providerId: form.providerId || null,
        model: form.model.trim(),
        temperature: form.temperature,
        maxTokens: form.maxTokens ? parseInt(form.maxTokens, 10) : null,
        skillIds: agent?.skillIds || [],
        knowledgeBaseIds: form.knowledgeBaseIds,
        channelBindings: form.channelBindings,
        isDefault: agent?.isDefault || false,
      };

      if (agent) {
        await updateAgent(agent.id, config);
      } else {
        await createAgent(config);
      }

      onClose();
      toast.success(tc('actions.save'));
    } catch {
      toast.error('Failed to save agent');
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={(v) => !v && onClose()}>
      <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {agent ? t('editor.editTitle') : t('editor.createTitle')}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* Basic Info */}
          <div className="space-y-4">
            <h3 className="text-sm font-medium">{t('editor.basicInfo')}</h3>

            <div className="grid grid-cols-[auto_1fr] gap-4 items-start">
              {/* Avatar */}
              <div className="space-y-2">
                <Label>{t('editor.avatar')}</Label>
                <div className="grid grid-cols-6 gap-1">
                  {EMOJI_OPTIONS.map((emoji) => (
                    <button
                      key={emoji}
                      type="button"
                      className={`h-9 w-9 rounded-md text-lg hover:bg-accent transition-colors ${
                        form.avatar === emoji ? 'bg-primary/20 ring-2 ring-primary' : ''
                      }`}
                      onClick={() => setForm((f) => ({ ...f, avatar: emoji }))}
                    >
                      {emoji}
                    </button>
                  ))}
                </div>
              </div>

              {/* Name & Description */}
              <div className="space-y-3">
                <div className="space-y-2">
                  <Label>{t('editor.name')}</Label>
                  <Input
                    value={form.name}
                    onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                    placeholder={t('editor.namePlaceholder')}
                  />
                </div>
                <div className="space-y-2">
                  <Label>{t('editor.description')}</Label>
                  <Input
                    value={form.description}
                    onChange={(e) => setForm((f) => ({ ...f, description: e.target.value }))}
                    placeholder={t('editor.descriptionPlaceholder')}
                  />
                </div>
              </div>
            </div>
          </div>

          {/* AI Configuration */}
          <div className="space-y-4">
            <h3 className="text-sm font-medium">{t('editor.aiConfig')}</h3>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label>{t('editor.provider')}</Label>
                <Select
                  value={form.providerId}
                  onChange={(e) => setForm((f) => ({ ...f, providerId: e.target.value }))}
                >
                  <option value="">{t('editor.providerDefault')}</option>
                  {providers.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
                </Select>
              </div>

              <div className="space-y-2">
                <Label>{t('editor.model')}</Label>
                <Input
                  value={form.model}
                  onChange={(e) => setForm((f) => ({ ...f, model: e.target.value }))}
                  placeholder={t('editor.modelPlaceholder')}
                />
              </div>

              <div className="space-y-2">
                <Label>{t('editor.temperature')}: {form.temperature.toFixed(1)}</Label>
                <input
                  type="range"
                  min="0"
                  max="2"
                  step="0.1"
                  value={form.temperature}
                  onChange={(e) => setForm((f) => ({ ...f, temperature: parseFloat(e.target.value) }))}
                  className="w-full accent-primary"
                />
              </div>

              <div className="space-y-2">
                <Label>{t('editor.maxTokens')}</Label>
                <Input
                  type="number"
                  value={form.maxTokens}
                  onChange={(e) => setForm((f) => ({ ...f, maxTokens: e.target.value }))}
                  placeholder={t('editor.maxTokensPlaceholder')}
                />
              </div>
            </div>
          </div>

          {/* System Prompt */}
          <div className="space-y-2">
            <Label>{t('editor.systemPrompt')}</Label>
            <Textarea
              value={form.systemPrompt}
              onChange={(e) => setForm((f) => ({ ...f, systemPrompt: e.target.value }))}
              placeholder={t('editor.systemPromptPlaceholder')}
              className="min-h-[160px] font-mono text-sm"
            />
          </div>

          {/* Knowledge Bases (F-3.4) */}
          <div className="space-y-3">
            <h3 className="text-sm font-medium">{t('editor.knowledgeBases')}</h3>
            <p className="text-xs text-muted-foreground">{t('editor.knowledgeBasesDesc')}</p>
            {knowledgeBases.length === 0 ? (
              <p className="text-sm text-muted-foreground">{t('editor.noKnowledgeBases')}</p>
            ) : (
              <div className="space-y-2">
                {knowledgeBases.map((kb) => (
                  <Checkbox
                    key={kb.id}
                    checked={form.knowledgeBaseIds.includes(kb.id)}
                    onChange={(e) => {
                      const checked = e.target.checked;
                      setForm((f) => ({
                        ...f,
                        knowledgeBaseIds: checked
                          ? [...f.knowledgeBaseIds, kb.id]
                          : f.knowledgeBaseIds.filter((id) => id !== kb.id),
                      }));
                    }}
                    label={`📚 ${kb.name} (${kb.documentCount} docs)`}
                  />
                ))}
              </div>
            )}
          </div>

          {/* Channel Bindings */}
          <div className="space-y-3">
            <h3 className="text-sm font-medium">{t('editor.channelBindings')}</h3>
            {channels.length === 0 ? (
              <p className="text-sm text-muted-foreground">{t('editor.noChannels')}</p>
            ) : (
              <div className="space-y-2">
                {channels.map((ch) => (
                  <Checkbox
                    key={ch.id}
                    checked={form.channelBindings.includes(ch.id)}
                    onChange={(e) => {
                      const checked = e.target.checked;
                      setForm((f) => ({
                        ...f,
                        channelBindings: checked
                          ? [...f.channelBindings, ch.id]
                          : f.channelBindings.filter((id) => id !== ch.id),
                      }));
                    }}
                    label={`${CHANNEL_ICONS[ch.type as ChannelType] || '📡'} ${ch.name || CHANNEL_NAMES[ch.type as ChannelType] || ch.type}`}
                  />
                ))}
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            {tc('actions.cancel')}
          </Button>
          <Button onClick={handleSave} disabled={saving}>
            {saving ? tc('status.saving') : tc('actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
