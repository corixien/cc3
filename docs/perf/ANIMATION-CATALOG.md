# Animationskatalog (automatisch erzeugt)

Erzeugt von `apps/cc3-desktop/scripts/perf/extract-animations.mjs`. Quelle fuer Phase 1.3 und 7 (native Nachbildung).
Fuer jede Animation muss eine Bildfolge bei 0/25/50/75/100 Prozent gegen die Referenz verglichen werden (siehe PLAN, Phase 1.3).

## @keyframes (11)

| Datei                | Name                          | Inhalt                                                                                                                                                                                                                         |
| -------------------- | ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| index.css            | t3-mobile-composer-old        | `0%, 35% { opacity: 1; } 65%, 100% { opacity: 0; } }`                                                                                                                                                                          |
| index.css            | t3-mobile-composer-new        | `0%, 35% { opacity: 0; } 65%, 100% { opacity: 1; } }`                                                                                                                                                                          |
| index.css            | t3-mobile-draft-headline-exit | `from { opacity: 1; transform: translateY(0); } to { opacity: 0; transform: translateY(-6px); } }`                                                                                                                             |
| index.css            | skeleton                      | `/* The single loading-bar breath used by every skeleton: one opacity pulse per container, stepped so however many bars sit under it, the compositor draws a handful of discrete frames per cycle rather than one per vsync —` |
| index.css            | live-tool-shine               | `from { background-position: -4.5rem 0, 0 0; } to { background-position: calc(100% + 4.5rem) 0, 0 0; } }`                                                                                                                      |
| index.css            | live-activity-focus           | `from { transform: translateX(0); } to { transform: translateX(100%); } }`                                                                                                                                                     |
| index.css            | live-activity-focus-counter   | `from { transform: translateX(0); } to { transform: translateX(-100%); } }`                                                                                                                                                    |
| index.css            | settings-search-target-pulse  | `0%, 100% { box-shadow: inset 0 0 0 0 transparent; } 50% { box-shadow: inset 0 0 0 2px color-mix(in oklab, var(--color-primary) 70%, transparent); } } [data-settings-page-scroll] div.settings-search-target-pulse, [data-se` |
| index.css            | preview-loading-progress      | `from { transform: scaleX(0.04); } to { transform: scaleX(0.9); } }`                                                                                                                                                           |
| index.css            | sidebar-drag-label-reveal     | `from { visibility: hidden; } to { visibility: visible; } }`                                                                                                                                                                   |
| pierre-tree-theme.ts | t3-tree-spin                  | `to { transform: rotate(360deg); } } `; export function pierreTreeStyle(colorScheme: "light" / "dark"): CSSProperties { return { colorScheme, ["--trees-fg-override" as string]: "var(--contrast-foreground)", }; }`           |

## Zeitfunktionen und Federn im Code

- components/BranchToolbar.tsx: cubic-bezier(0.32, 0.72, 0, 1)
- components/Sidebar.motion.ts: duration: 150
- components/chat/draftHeroTransition.ts: cubic-bezier(0.4, 0, 0.2, 1)
- components/chat/useComposerRestingTransition.ts: cubic-bezier(0.32, 0.72, 0, 1)
- components/ui/toast.tsx: cubic-bezier(.22,1,.36,1)
- index.css: cubic-bezier(0.1, 0.5, 0.2, 1)
- index.css: cubic-bezier(0.32, 0.72, 0, 1)
- index.css: cubic-bezier(0.4, 0, 0.2, 1)
- index.css: cubic-bezier(0.4, 0, 1, 1)
- index.css: duration: 0

## Tailwind-Klassen fuer Animation/Transition je Datei (171 Dateien)

- `browser/BrowserViewportResizeHandles.tsx`: `duration-150` `transition-colors`
- `components/AnimatedHeight.tsx`: `duration-200` `ease-out` `motion-reduce:transition-none` `transition` `transition-[height]`
- `components/AnimatedWidth.tsx`: `duration-150` `ease-out` `motion-reduce:transition-none` `transition-[width]`
- `components/AppRouteModal.tsx`: `duration-150` `ease-out` `transition-[opacity`
- `components/ChatView.tsx`: `motion-safe:ease-out` `motion-safe:transition-[padding-left]` `transition`
- `components/ComposerContextLabel.tsx`: `duration-180` `ease-drawer` `motion-reduce:transition-none` `transition-opacity`
- `components/ConnectionStatusDot.tsx`: `duration-2000` `motion-safe:animate-status-ping`
- `components/ContextChip.tsx`: `motion-reduce:transition-none` `transition-colors`
- `components/GitActionsControl.tsx`: `duration-200` `ease-out` `motion-reduce:transition-none` `transition-[background-color` `transition-[grid-template-rows` `transition-colors` `transition-transform`
- `components/HistorySwipeNavigation.tsx`: `duration-100` `duration-200` `ease-out` `transition-[opacity` `transition-opacity`
- `components/PersistentThreadTerminals.tsx`: `ease-out` `transition-[grid-template-rows]`
- `components/ProjectScriptsControl.tsx`: `transition-opacity`
- `components/RightPanelSheet.tsx`: `transition`
- `components/RightPanelTabs.tsx`: `transition` `transition-colors`
- `components/Sidebar.motion.ts`: `ease-out` `transition`
- `components/Sidebar.tsx`: `transition-colors`
- `components/ThreadNotificationCoordinator.tsx`: `transition`
- `components/ThreadStatusIndicators.tsx`: `motion-safe:animate-status-pulse` `transition-colors`
- `components/ThreadTerminalDrawer.tsx`: `transition-colors`
- `components/WorkspacePageHeader.tsx`: `motion-safe:ease-out` `motion-safe:transition-[padding-left`
- `components/auth/ConnectAgentSurface.tsx`: `transition-[background-color`
- `components/chat/AssistantCitationSource.tsx`: `ease-in-out`
- `components/chat/ChangedFilesTree.tsx`: `transition-colors` `transition-transform`
- `components/chat/ChatCanvas.tsx`: `transition`
- `components/chat/ChatComposer.tsx`: `duration-200` `transition-[background-color]` `transition-actions` `transition-prompt`
- `components/chat/ChatHeader.tsx`: `transition-colors` `transition-opacity`
- `components/chat/ComposerAttachmentStrip.tsx`: `duration-300` `ease-drawer` `motion-reduce:transition-none` `transition-[opacity` `transition-opacity`
- `components/chat/ComposerBannerStack.tsx`: `duration-150` `ease-out` `motion-reduce:transition-none` `transition-opacity`
- `components/chat/ComposerPendingUserInputPanel.tsx`: `duration-150` `transition-colors`
- `components/chat/ComposerPrimaryActions.tsx`: `duration-150` `transition-actions` `transition-all` `transition-colors`
- `components/chat/ComposerStashBadge.tsx`: `animate-stash-pulse-count` `animate-stash-pulse-row` `animate-stash-pulse-row-alt` `duration-180` `duration-200` `ease-out` `motion-reduce:transition-none` `transition-[opacity` `transition-colors`
- `components/chat/DraftHeroHeadline.tsx`: `transition-colors`
- `components/chat/ExpandedImageDialog.tsx`: `duration-140` `ease-out` `motion-reduce:transition-none` `transition-opacity`
- `components/chat/HtmlRenderFrame.tsx`: `duration-150` `transition-opacity`
- `components/chat/MarkdownDetails.tsx`: `transition-transform`
- `components/chat/MessagesTimeline.tsx`: `transition-colors`
- `components/chat/ModelPickerSidebar.tsx`: `duration-200` `ease-out` `transition-[top]` `transition-colors`
- `components/chat/PanelLayoutControls.tsx`: `motion-safe:ease-out` `motion-safe:transition-opacity` `transition`
- `components/chat/QueuedRunsControl.tsx`: `duration-150` `duration-200` `motion-reduce:transition-none` `transition-opacity` `transition-transform`
- `components/chat/ThreadDetailsCard.tsx`: `duration-150` `ease-out` `motion-reduce:transition-none` `transition` `transition-[top]`
- `components/chat/ThreadDetailsControl.tsx`: `transition-[box-shadow`
- `components/chat/ThreadFindBar.tsx`: `duration-150` `ease-out` `motion-reduce:transition-none` `transition-opacity`
- `components/chat/ThreadLineageWorkflowRow.tsx`: `transition-transform`
- `components/chat/ThreadNow.tsx`: `duration-150` `ease-out` `motion-reduce:transition-none` `transition-opacity`
- `components/chat/ThreadRelationshipsControl.tsx`: `transition-opacity` `transition-transform`
- `components/chat/TimelineAssistantRows.tsx`: `animate-pulse` `duration-200` `transition-opacity`
- `components/chat/TimelineMinimap.tsx`: `after:duration-150` `after:transition-opacity` `duration-150` `transition-opacity` `transition-transform`
- `components/chat/TimelineSystemDivider.tsx`: `transition-colors`
- `components/chat/TimelineUserMessageRow.tsx`: `duration-200` `transition-opacity` `transition-transform`
- `components/chat/TimelineV2EventRow.tsx`: `transition-opacity` `transition-transform`
- `components/chat/TimelineWorkEntryRow.tsx`: `duration-200` `transition-transform`
- `components/chat/V2LifecycleRow.tsx`: `transition-colors`
- `components/chat/VoiceInputPill.tsx`: `duration-100` `ease-out` `motion-reduce:transition-none` `transition-[transform`
- `components/chat/VoiceInputSetup.tsx`: `duration-200` `transition-[width]`
- `components/chat/WorkLog.tsx`: `transition-colors`
- `components/chat/WorkflowOpeningBlock.tsx`: `transition-colors`
- `components/chat/chatCanvasLayout.ts`: `transition`
- `components/chat/draftHeroTransition.ts`: `transition` `transition-duration`
- `components/chat/threadDetailsCardLayout.ts`: `transition`
- `components/chat/timelineViewport.ts`: `transition`
- `components/chat/useComposerRestingTransition.ts`: `transition` `transition-actions` `transition-prompt`
- `components/chat/useComposerSend.ts`: `transition`
- `components/chat/useDraftHeroLayoutTransition.ts`: `transition`
- `components/chat/useTimelineFollow.ts`: `transition`
- `components/cloud/CloudEnvironmentConnectList.tsx`: `duration-2000`
- `components/desktopUpdate.toast.tsx`: `transition-colors`
- `components/device/DeviceStreamView.tsx`: `delay-2000` `duration-0` `transition-opacity`
- `components/device/DeviceToolsPanel.tsx`: `transition-transform`
- `components/diffs/DiffCommentAnnotation.tsx`: `transition-opacity`
- `components/diffs/StyledDiffCodeView.tsx`: `ease-out` `transition`
- `components/files/MermaidViewer.tsx`: `animate-none`
- `components/files/officeArchive.ts`: `transition`
- `components/onboarding/FirstRunGate.tsx`: `transition`
- `components/onboarding/WelcomePage.tsx`: `transition-transform`
- `components/preview/AgentBrowserCursor.tsx`: `animate-status-ping` `duration-150` `ease-out` `motion-reduce:animate-none` `motion-reduce:transition-none` `transition-[opacity`
- `components/preview/PreviewChromeRow.tsx`: `motion-safe:animate-status-pulse` `transition-opacity`
- `components/preview/PreviewPanelShell.tsx`: `ease-out` `transition` `transition-[width]`
- `components/preview/RightPanelResizeHandle.tsx`: `duration-150` `transition-colors`
- `components/preview/ThreadPreviewMiniPlayer.tsx`: `motion-safe:animate-status-pulse` `transition-opacity`
- `components/preview/ZoomIndicator.tsx`: `duration-200` `ease-out` `transition-all`
- `components/pullRequest/PullRequestCodeTab.tsx`: `transition-transform`
- `components/pullRequest/PullRequestCopyableCode.tsx`: `duration-150` `motion-reduce:transition-none` `transition-colors` `transition-opacity`
- `components/pullRequest/PullRequestDetailCrumbs.tsx`: `delay-50` `duration-100` `duration-150` `ease-out` `motion-reduce:transition-none` `transition-[opacity`
- `components/pullRequest/PullRequestDetailPanel.tsx`: `delay-50` `duration-100` `duration-150` `duration-200` `ease-out` `motion-reduce:transition-none` `transition-[grid-template-rows]` `transition-[opacity` `transition-opacity`
- `components/pullRequest/PullRequestEditButton.tsx`: `motion-reduce:transition-none` `transition-opacity`
- `components/pullRequest/PullRequestGhosts.tsx`: `motion-safe:animate-skeleton`
- `components/pullRequest/PullRequestReactions.tsx`: `transition-colors`
- `components/pullRequest/PullRequestRow.tsx`: `transition-colors`
- `components/pullRequest/PullRequestSummaryTab.tsx`: `transition-transform`
- `components/pullRequest/PullRequestTimelineTab.tsx`: `transition-colors` `transition-opacity` `transition-transform`
- `components/pullRequest/PullRequestsPage.tsx`: `ease-out` `transition` `transition-[width`
- `components/settings/AddProviderInstanceDialog.tsx`: `transition-transform`
- `components/settings/AdvancedSettings.tsx`: `duration-150` `motion-reduce:transition-none` `transition-colors` `transition-transform`
- `components/settings/AuthorizedClientsSettings.tsx`: `duration-2000` `transition-colors`
- `components/settings/BrowserImportWizard.tsx`: `transition-colors`
- `components/settings/ConnectionDiagnosisDetails.tsx`: `duration-150` `motion-reduce:transition-none` `transition-transform`
- `components/settings/DiagnosticsSettings.tsx`: `transition-colors`
- `components/settings/EnvironmentRoutesList.tsx`: `transition`
- `components/settings/FoldedSettingsSection.tsx`: `duration-150` `motion-reduce:transition-none` `transition-transform`
- `components/settings/FontFamilyPicker.tsx`: `transition`
- `components/settings/KeybindingsSettings.tsx`: `transition-colors` `transition-opacity`
- `components/settings/MicrophoneOrderList.tsx`: `transition`
- `components/settings/OpenSourceLicenses.tsx`: `duration-200` `transition-transform`
- `components/settings/PanelAnimationsPreview.tsx`: `ease-out` `motion-reduce:transition-none` `transition` `transition-[width`
- `components/settings/ProviderInstanceCard.tsx`: `transition-colors` `transition-opacity`
- `components/settings/ProviderModelsSection.tsx`: `transition-colors`
- `components/settings/ProviderSettingsPanel.tsx`: `transition-colors`
- `components/settings/RedactedSensitiveText.tsx`: `transition`
- `components/settings/ResourceTelemetryDiagnostics.tsx`: `transition-colors`
- `components/settings/SavedEnvironmentRows.tsx`: `duration-150` `motion-reduce:transition-none` `transition-transform`
- `components/settings/SourceControlSettings.tsx`: `transition-colors` `transition-transform`
- `components/settings/ThemeColorPicker.tsx`: `transition-[background-color` `transition-transform`
- `components/settings/ThemeImportDialog.tsx`: `transition-colors`
- `components/settings/ThemePreviewCircles.tsx`: `transition-transform`
- `components/settings/ThemeSearchSection.tsx`: `transition-colors`
- `components/settings/ThemeSettings.tsx`: `duration-200` `ease-out` `motion-reduce:transition-none` `transition` `transition-[opacity` `transition-colors`
- `components/settings/VoiceSettingsPanel.tsx`: `duration-200` `transition-[width]`
- `components/settings/settingsLayout.tsx`: `transition`
- `components/settings/themeInspector.ts`: `transition`
- `components/sidebar/DesktopUpdateStatusIcon.tsx`: `duration-300` `ease-out` `motion-reduce:transition-none` `transition-[stroke-dashoffset]`
- `components/sidebar/SidebarDraftBlock.tsx`: `transition-opacity`
- `components/sidebar/SidebarListMarkers.tsx`: `transition`
- `components/sidebar/SidebarProjectFolderHeader.tsx`: `transition-transform`
- `components/sidebar/SidebarProviderUpdatePill.tsx`: `duration-180` `ease-drawer` `ease-linear` `transition` `transition-all` `transition-colors` `transition-transform`
- `components/sidebar/SidebarThreadRow.tsx`: `ease-out` `motion-reduce:transition-none` `transition` `transition-colors` `transition-opacity`
- `components/sidebar/SidebarThreadUndoNotice.tsx`: `duration-200` `ease-out` `transition` `transition-opacity`
- `components/sidebar/SidebarUpdatePill.tsx`: `transition-colors`
- `components/sidebar/SidebarUpdateReleaseNotes.tsx`: `transition-colors`
- `components/sidebar/UpdateEverythingDialog.tsx`: `transition-transform`
- `components/ui/autocomplete.tsx`: `transition-[color` `transition-[scale` `transition-colors`
- `components/ui/badge.tsx`: `transition-shadow`
- `components/ui/button.tsx`: `transition-[box-shadow`
- `components/ui/calendar.tsx`: `transition-[border-radius`
- `components/ui/checkbox.tsx`: `transition-shadow`
- `components/ui/collapsible-section-header.tsx`: `transition-transform`
- `components/ui/collapsible.tsx`: `duration-200` `motion-reduce:transition-none` `transition-[height]`
- `components/ui/color-picker.tsx`: `transition`
- `components/ui/combobox.tsx`: `transition-[scale` `transition-colors` `transition-opacity`
- `components/ui/dialog-styles.ts`: `duration-200` `ease-in-out` `transition-[scale` `transition-all`
- `components/ui/input-group.tsx`: `transition-shadow`
- `components/ui/input.tsx`: `transition` `transition-shadow`
- `components/ui/menu.tsx`: `duration-200` `transition` `transition-[background-color` `transition-colors`
- `components/ui/number-field.tsx`: `transition` `transition-colors` `transition-shadow`
- `components/ui/popover.tsx`: `data-current:transition-opacity` `data-instant:transition-[top]` `data-instant:transition-none` `data-previous:transition-opacity` `duration-150` `ease-out` `motion-reduce:data-instant:transition-none` `motion-reduce:transition-none` `transition` `transition-[top]` `transition-[width` `transition-transform`
- `components/ui/preview-card.tsx`: `transition-[scale`
- `components/ui/radio-group.tsx`: `transition-shadow`
- `components/ui/refresh-icon.tsx`: `animate-spin`
- `components/ui/scroll-area.tsx`: `data-hovering:delay-0` `data-hovering:duration-100` `data-scrolling:delay-0` `data-scrolling:duration-100` `delay-300` `transition-colors` `transition-opacity` `transition-shadow`
- `components/ui/select.tsx`: `transition-[color`
- `components/ui/sheet.tsx`: `duration-200` `ease-in-out` `transition` `transition-[opacity` `transition-all` `transition-none`
- `components/ui/sidebar.tsx`: `ease-out` `transition` `transition-[left` `transition-[width` `transition-[width]` `transition-all` `transition-duration`
- `components/ui/skeleton.tsx`: `motion-safe:animate-skeleton`
- `components/ui/spinner.tsx`: `animate-spin`
- `components/ui/switch.tsx`: `duration-200` `transition` `transition-[background-color`
- `components/ui/table.tsx`: `transition-colors`
- `components/ui/textarea.tsx`: `transition-shadow`
- `components/ui/toast.logic.ts`: `transition`
- `components/ui/toast.tsx`: `duration-250` `transition` `transition-[color` `transition-[scale` `transition-colors` `transition-opacity`
- `components/ui/toggle.tsx`: `transition-colors` `transition-shadow`
- `components/ui/tooltip.tsx`: `data-instant:duration-0` `data-instant:transition-none` `transition-[scale` `transition-[top`
- `components/usage/UsageLimitsPooled.tsx`: `transition`
- `components/usage/UsagePage.tsx`: `delay-150` `transition-colors` `transition-opacity`
- `components/usage/UsageProviderChart.tsx`: `delay-150` `transition-opacity`
- `composerDraftStore.ts`: `transition`
- `contextMenuFallback.ts`: `transition-colors`
- `hooks/useResizableWidth.ts`: `transition` `transition-duration.` `transition-property`
- `hooks/useTheme.ts`: `transition`
- `index.css`: `animate-ping` `animate-skeleton` `animate-spin` `animate-stash-pulse-count` `animate-stash-pulse-row` `animate-stash-pulse-row-alt` `animate-status-ping` `animate-status-pulse` `ease-drawer` `ease-in-out` `ease-out` `ease-out,` `transition` `transition-duration` `transition-group` `transition-image-pair` `transition-new` `transition-old`
- `onboarding/firstRun.logic.ts`: `transition`
- `panelAnimations.ts`: `transition`
- `terminal/ghostty/surface.ts`: `duration-[120ms]` `ease-[ease-out]` `transition` `transition-[background-color]`
