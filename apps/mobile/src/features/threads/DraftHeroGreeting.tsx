// CC3: the empty-chat greeting of the desktop. A plain line that cycles; nothing to press.
import { useGT } from "gt-react-native";
import { useEffect, useState } from "react";

import { AppText as Text } from "../../components/ui/AppText";

const CYCLE_MS = 6000;

export function DraftHeroGreeting() {
  const gt = useGT();
  const greetings = [
    gt("What should we work on?"),
    gt("What's up?"),
    gt("Ready when you are."),
    gt("What are we building today?"),
    gt("How can I help?"),
    gt("Where should we start?"),
  ];
  const [index, setIndex] = useState(0);
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- cycles the greeting on a timer
  useEffect(() => {
    const timer = setInterval(() => setIndex((current) => (current + 1) % 6), CYCLE_MS);
    return () => clearInterval(timer);
  }, []);
  return (
    <Text className="text-center text-3xl font-t3-medium tracking-tight text-foreground">
      {greetings[index]}
    </Text>
  );
}
