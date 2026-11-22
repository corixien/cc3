// CC3: header buttons of both split-view chats: drop the left chat or drop the right chat.
import { useNavigate } from "@tanstack/react-router";
import { useGT } from "gt-react";
import { PanelLeftCloseIcon, PanelRightCloseIcon } from "lucide-react";

import { useSplitViewStore } from "../../splitViewStore";
import { buildThreadRouteParams } from "../../threadRoutes";
import { Toggle } from "../ui/toggle";
import { Tooltip, TooltipPopup, TooltipTrigger } from "../ui/tooltip";

export function SplitPaneControls() {
  const gt = useGT();
  const navigate = useNavigate();
  const secondary = useSplitViewStore((state) => state.secondary);
  const close = useSplitViewStore((state) => state.close);
  if (secondary === null) return null;

  const control = (label: string, onClick: () => void, icon: React.ReactNode) => (
    <Tooltip>
      <TooltipTrigger render={<span className="flex shrink-0" />}>
        <Toggle
          className="shrink-0 [-webkit-app-region:no-drag]"
          pressed={false}
          onPressedChange={onClick}
          aria-label={label}
          variant="ghost"
          size="sm"
        >
          {icon}
        </Toggle>
      </TooltipTrigger>
      <TooltipPopup side="bottom">{label}</TooltipPopup>
    </Tooltip>
  );
  return (
    <>
      {control(
        gt("Remove the left chat"),
        () => {
          close();
          void navigate({
            to: "/$environmentId/$threadId",
            params: buildThreadRouteParams(secondary),
          });
        },
        <PanelLeftCloseIcon className="size-4" />,
      )}
      {control(gt("Remove the right chat"), close, <PanelRightCloseIcon className="size-4" />)}
    </>
  );
}
