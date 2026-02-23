/**
 * Chat Toolbar
 * Agent selector, session selector, new session, refresh, and thinking toggle.
 * Rendered in the Header when on the Chat page.
 */
import { useEffect } from 'react';
import { RefreshCw, Brain, ChevronDown, Plus, Filter } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useChatStore } from '@/stores/chat';
import { useAgentStore } from '@/stores/agents';
import { cn } from '@/lib/utils';
import { useTranslation } from 'react-i18next';

export function ChatToolbar() {
  const sessions = useChatStore((s) => s.sessions);
  const currentSessionKey = useChatStore((s) => s.currentSessionKey);
  const switchSession = useChatStore((s) => s.switchSession);
  const newSession = useChatStore((s) => s.newSession);
  const refresh = useChatStore((s) => s.refresh);
  const loading = useChatStore((s) => s.loading);
  const showThinking = useChatStore((s) => s.showThinking);
  const toggleThinking = useChatStore((s) => s.toggleThinking);
  const agentSessionFilter = useChatStore((s) => s.agentSessionFilter);
  const setAgentSessionFilter = useChatStore((s) => s.setAgentSessionFilter);
  const { t } = useTranslation('chat');
  const { t: ta } = useTranslation('agents');

  const agents = useAgentStore((s) => s.agents);
  const activeAgentId = useAgentStore((s) => s.activeAgentId);
  const setActiveAgent = useAgentStore((s) => s.setActiveAgent);
  const getActiveAgent = useAgentStore((s) => s.getActiveAgent);
  const fetchAgents = useAgentStore((s) => s.fetchAgents);

  useEffect(() => {
    fetchAgents();
  }, [fetchAgents]);

  // Sync filter with active agent on mount
  useEffect(() => {
    if (activeAgentId && !agentSessionFilter) {
      setAgentSessionFilter(activeAgentId);
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const activeAgent = getActiveAgent();
  const isFiltering = !!agentSessionFilter;

  const handleSessionChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    switchSession(e.target.value);
  };

  const handleAgentChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const id = e.target.value;
    setActiveAgent(id);
    setAgentSessionFilter(id);
  };

  const toggleFilter = () => {
    if (agentSessionFilter) {
      setAgentSessionFilter(null);
    } else if (activeAgentId) {
      setAgentSessionFilter(activeAgentId);
    }
  };

  return (
    <div className="flex items-center gap-2">
      {/* Agent Selector */}
      {agents.length > 0 && (
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="relative">
              <select
                value={activeAgentId || ''}
                onChange={handleAgentChange}
                className={cn(
                  'appearance-none rounded-md border border-border bg-background px-3 py-1.5 pr-8 pl-8',
                  'text-sm text-foreground cursor-pointer',
                  'focus:outline-none focus:ring-2 focus:ring-ring',
                )}
              >
                {agents.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.name}
                  </option>
                ))}
              </select>
              <span className="absolute left-2 top-1/2 -translate-y-1/2 text-base pointer-events-none">
                {activeAgent?.avatar || '🤖'}
              </span>
              <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground pointer-events-none" />
            </div>
          </TooltipTrigger>
          <TooltipContent>
            <p>{ta('toolbar.switchAgent')}</p>
          </TooltipContent>
        </Tooltip>
      )}

      {/* Session Selector */}
      <div className="relative">
        <select
          value={currentSessionKey}
          onChange={handleSessionChange}
          className={cn(
            'appearance-none rounded-md border border-border bg-background px-3 py-1.5 pr-8',
            'text-sm text-foreground cursor-pointer',
            'focus:outline-none focus:ring-2 focus:ring-ring',
          )}
        >
          {/* Render all sessions; if currentSessionKey is not in the list, add it */}
          {!sessions.some((s) => s.key === currentSessionKey) && (
            <option value={currentSessionKey}>
              {currentSessionKey}
            </option>
          )}
          {sessions.map((s) => (
            <option key={s.key} value={s.key}>
              {s.key}
            </option>
          ))}
        </select>
        <ChevronDown className="absolute right-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground pointer-events-none" />
      </div>

      {/* New Session */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8"
            onClick={newSession}
          >
            <Plus className="h-4 w-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <p>{t('toolbar.newSession')}</p>
        </TooltipContent>
      </Tooltip>

      {/* Filter by Agent */}
      {agents.length > 0 && (
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className={cn(
                'h-8 w-8',
                isFiltering && 'bg-primary/10 text-primary',
              )}
              onClick={toggleFilter}
            >
              <Filter className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>
            <p>{isFiltering ? t('toolbar.showAllSessions') : t('toolbar.filterByAgent')}</p>
          </TooltipContent>
        </Tooltip>
      )}

      {/* Refresh */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8"
            onClick={() => refresh()}
            disabled={loading}
          >
            <RefreshCw className={cn('h-4 w-4', loading && 'animate-spin')} />
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <p>{t('toolbar.refresh')}</p>
        </TooltipContent>
      </Tooltip>

      {/* Thinking Toggle */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className={cn(
              'h-8 w-8',
              showThinking && 'bg-primary/10 text-primary',
            )}
            onClick={toggleThinking}
          >
            <Brain className="h-4 w-4" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <p>{showThinking ? t('toolbar.hideThinking') : t('toolbar.showThinking')}</p>
        </TooltipContent>
      </Tooltip>
    </div>
  );
}
