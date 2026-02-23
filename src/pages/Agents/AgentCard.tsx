/**
 * Agent Card
 * Displays a single agent in the grid with actions dropdown
 */
import { MoreHorizontal, Copy, Trash2, Star, Download } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useTranslation } from 'react-i18next';
import type { AgentConfig } from '@/types/agent';
import { useChannelsStore } from '@/stores/channels';
import { CHANNEL_ICONS, type ChannelType } from '@/types/channel';

interface AgentCardProps {
  agent: AgentConfig;
  isActive: boolean;
  onEdit: (agent: AgentConfig) => void;
  onClone: (id: string) => void;
  onDelete: (agent: AgentConfig) => void;
  onSetDefault: (id: string) => void;
  onExport: (id: string) => void;
  onSelect: (id: string) => void;
}

export function AgentCard({
  agent,
  isActive,
  onEdit,
  onClone,
  onDelete,
  onSetDefault,
  onExport,
  onSelect,
}: AgentCardProps) {
  const { t } = useTranslation('agents');
  const allChannels = useChannelsStore((s) => s.channels);

  const boundChannels = allChannels.filter((ch) =>
    agent.channelBindings?.includes(ch.id)
  );

  return (
    <Card
      className={`cursor-pointer transition-all hover:shadow-md ${isActive ? 'ring-2 ring-primary' : ''}`}
      onClick={() => onSelect(agent.id)}
    >
      <CardContent className="p-4">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3 min-w-0">
            <span className="text-3xl shrink-0">{agent.avatar}</span>
            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <h3 className="font-semibold truncate">{agent.name}</h3>
                {agent.isDefault && (
                  <Badge variant="secondary" className="shrink-0 text-xs">
                    {t('card.default')}
                  </Badge>
                )}
              </div>
              <p className="text-sm text-muted-foreground truncate mt-0.5">
                {agent.description || t('card.noDescription')}
              </p>
            </div>
          </div>

          <DropdownMenu>
            <DropdownMenuTrigger asChild onClick={(e) => e.stopPropagation()}>
              <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" onClick={(e) => e.stopPropagation()}>
              <DropdownMenuItem onClick={() => onEdit(agent)}>
                {t('actions.edit')}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => onClone(agent.id)}>
                <Copy className="h-4 w-4 mr-2" />
                {t('actions.clone')}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => onExport(agent.id)}>
                <Download className="h-4 w-4 mr-2" />
                {t('actions.export')}
              </DropdownMenuItem>
              {!agent.isDefault && (
                <>
                  <DropdownMenuItem onClick={() => onSetDefault(agent.id)}>
                    <Star className="h-4 w-4 mr-2" />
                    {t('actions.setDefault')}
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    className="text-destructive"
                    onClick={() => onDelete(agent)}
                  >
                    <Trash2 className="h-4 w-4 mr-2" />
                    {t('actions.delete')}
                  </DropdownMenuItem>
                </>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        <div className="mt-3 flex items-center gap-2 flex-wrap">
          {agent.model && (
            <Badge variant="outline" className="text-xs">
              {agent.model}
            </Badge>
          )}
          {agent.skillIds.length > 0 && (
            <Badge variant="outline" className="text-xs">
              {t('card.skills', { count: agent.skillIds.length })}
            </Badge>
          )}
          {boundChannels.length > 0 && (
            <Badge variant="outline" className="text-xs">
              {boundChannels.map((ch) => CHANNEL_ICONS[ch.type as ChannelType] || '📡').join('')}{' '}
              {t('card.channels', { count: boundChannels.length })}
            </Badge>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
