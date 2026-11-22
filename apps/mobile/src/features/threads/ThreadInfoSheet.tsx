// CC3: the thread info card of the desktop, as a sheet. Repository and branch, working folder,
// context window with its breakdown, MCP servers, background work and the account usage.
import { useNavigation, type StaticScreenProps } from "@react-navigation/native";
import { scopeProjectRef, scopeThreadRef } from "@t3tools/client-runtime/environment";
import {
  deriveLatestContextWindowSnapshot,
  latestProviderTurnTokenUsage,
} from "@t3tools/client-runtime/state/context-window";
import { presentPendingBackgroundWork } from "@t3tools/client-runtime/state/thread-execution";
import type { EnvironmentId, ThreadId } from "@t3tools/contracts";
import { T, useGT, useMessages } from "gt-react-native";
import type { ReactNode } from "react";
import { Pressable, ScrollView, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";

import { SymbolView } from "../../components/ui/AppSymbol";
import { AppText as Text } from "../../components/ui/AppText";
import { ScreenContent } from "../../components/ui/ScreenContent";
import { SheetHeader } from "../../components/ui/SheetHeader";
import { useProject, useThreadShell } from "../../state/entities";
import { useThreadProjection, useThreadVisibleTurnItems } from "../../state/use-thread-detail";

type InfoTarget = { readonly environmentId: EnvironmentId; readonly threadId: ThreadId };

function formatTokens(value: number | null): string {
  if (value === null || !Number.isFinite(value)) return "0";
  if (value < 1_000) return `${Math.round(value)}`;
  if (value < 10_000) return `${(value / 1_000).toFixed(1).replace(/\.0$/, "")}k`;
  if (value < 999_500) return `${Math.round(value / 1_000)}k`;
  return `${(value / 1_000_000).toFixed(1).replace(/\.0$/, "")}m`;
}

const home = (path: string) => path.replace(/^\/home\/[^/]+/, "~");

function Section(props: { readonly title: string; readonly children: ReactNode }) {
  return (
    <View className="gap-1.5 pt-5">
      <Text className="text-xs font-t3-medium uppercase tracking-wider text-foreground-muted">
        {props.title}
      </Text>
      {props.children}
    </View>
  );
}

function Row(props: {
  readonly icon: "folder" | "chevron.left.forwardslash.chevron.right" | "terminal" | "clock";
  readonly children: ReactNode;
  readonly detail?: string;
}) {
  return (
    <View className="flex-row items-center gap-3 py-2">
      <SymbolView
        name={props.icon}
        size={15}
        tintColorClassName="accent-foreground-muted"
        type="monochrome"
      />
      <Text className="min-w-0 flex-1 text-sm text-foreground" numberOfLines={2}>
        {props.children}
      </Text>
      {props.detail ? (
        <Text className="text-xs tabular-nums text-foreground-muted">{props.detail}</Text>
      ) : null}
    </View>
  );
}

export function ThreadInfoSheet({ route }: StaticScreenProps<InfoTarget>) {
  const navigation = useNavigation();
  const insets = useSafeAreaInsets();
  const gt = useGT();
  const m = useMessages();
  const { environmentId, threadId } = route.params;
  const shell = useThreadShell(scopeThreadRef(environmentId, threadId));
  const project = useProject(shell ? scopeProjectRef(environmentId, shell.projectId) : null);
  const detail = useThreadProjection({ environmentId, threadId });
  const items = useThreadVisibleTurnItems({ environmentId, threadId });
  const projection = detail?.projection ?? null;
  const providerThread = projection?.providerThreads.find(
    (thread) => thread.id === projection.thread.activeProviderThreadId,
  );
  const liveUsage = latestProviderTurnTokenUsage(
    projection?.providerTurns ?? [],
    projection,
    providerThread,
  );
  const context = deriveLatestContextWindowSnapshot(items, liveUsage, providerThread);
  const max = context?.maxTokens != null && context.maxTokens > 0 ? context.maxTokens : null;
  const used = context?.usedTokens ?? 0;
  const usedPercent = max !== null ? Math.min(100, Math.max(0, (used / max) * 100)) : null;
  const categories = [...(context?.contextCategories ?? [])].sort((a, b) => {
    const order = { used: 0, buffer: 1, free: 2, deferred: 3 } as const;
    return order[a.kind] - order[b.kind] || b.tokens - a.tokens;
  });
  const background = presentPendingBackgroundWork(shell?.pendingBackgroundTasks ?? [], m);
  const folder = shell?.worktreePath ?? project?.workspaceRoot ?? null;
  const repository = project?.title ?? null;

  return (
    <View collapsable={false} className="flex-1 bg-sheet">
      <SheetHeader title={gt("Thread details")} onClose={() => navigation.goBack()} />
      <ScreenContent>
        <ScrollView
          nestedScrollEnabled
          className="flex-1"
          contentContainerClassName="px-5"
          contentContainerStyle={{ paddingBottom: Math.max(insets.bottom, 16) + 8 }}
        >
          <Section title={gt("Repository")}>
            <Row icon="chevron.left.forwardslash.chevron.right">
              {repository
                ? `${repository}${shell?.branch ? ` / ${shell.branch}` : ""}`
                : gt("No repo")}
            </Row>
            {folder ? <Row icon="folder">{home(folder)}</Row> : null}
          </Section>

          {context ? (
            <Section title={gt("Context")}>
              <View className="gap-1.5 py-1">
                <View className="flex-row items-baseline justify-between">
                  <Text className="text-sm text-foreground">
                    {usedPercent !== null
                      ? gt("{percent}% used", { percent: Math.round(usedPercent) })
                      : gt("Context")}
                  </Text>
                  <Text className="text-xs tabular-nums text-foreground-muted">
                    {formatTokens(used)}
                    {max !== null ? ` / ${formatTokens(max)}` : ""}
                  </Text>
                </View>
                {usedPercent !== null ? (
                  <View className="h-1.5 flex-row overflow-hidden rounded-full bg-subtle">
                    <View
                      className={
                        usedPercent >= 90
                          ? "h-full bg-red-500"
                          : usedPercent >= 70
                            ? "h-full bg-amber-500"
                            : "h-full bg-foreground"
                      }
                      style={{ flex: usedPercent }}
                    />
                    <View style={{ flex: 100 - usedPercent }} />
                  </View>
                ) : null}
              </View>
              {categories.map((category) => (
                <View key={`${category.kind}:${category.name}`} className="flex-row gap-3 py-0.5">
                  <Text className="min-w-0 flex-1 text-xs text-foreground-muted" numberOfLines={1}>
                    {category.name}
                  </Text>
                  <Text className="text-xs tabular-nums text-foreground">
                    {formatTokens(category.tokens)}
                  </Text>
                  {max !== null ? (
                    <Text className="w-12 text-right text-xs tabular-nums text-foreground-muted">
                      {((category.tokens / max) * 100).toFixed(1)}%
                    </Text>
                  ) : null}
                </View>
              ))}
            </Section>
          ) : null}

          {context?.mcpServers && context.mcpServers.length > 0 ? (
            <Section title={gt("MCP servers")}>
              {context.mcpServers.map((server) => (
                <View key={server.name} className="flex-row gap-3 py-1">
                  <Text className="min-w-0 flex-1 text-sm text-foreground" numberOfLines={1}>
                    {server.name}
                  </Text>
                  <Text className="text-xs text-foreground-muted">{server.status}</Text>
                </View>
              ))}
            </Section>
          ) : null}

          {background !== null ? (
            <Section title={gt("Background work")}>
              {background.items.map((item) => (
                <Row key={item.taskId} icon={item.kind === "monitor" ? "clock" : "terminal"}>
                  {item.label}
                </Row>
              ))}
            </Section>
          ) : null}

          <Section title={gt("Usage")}>
            <Pressable
              accessibilityRole="button"
              onPress={() => {
                navigation.goBack();
                navigation.navigate("SettingsUsage" as never);
              }}
              className="flex-row items-center justify-between py-2 active:opacity-70"
            >
              <Text className="text-sm text-foreground">
                <T>Open usage limits</T>
              </Text>
              <SymbolView name="chevron.right" size={12} tintColorClassName="accent-icon-subtle" />
            </Pressable>
          </Section>
        </ScrollView>
      </ScreenContent>
    </View>
  );
}
