// CC3: which local folder belongs to which GitHub repository, kept across restarts.
import { create } from "zustand";
import { persist } from "zustand/middleware";

interface RepoFolderLinks {
  readonly links: Readonly<Record<string, string>>;
  readonly link: (repo: string, folder: string) => void;
  readonly unlink: (repo: string) => void;
}

export const useRepoFolderLinks = create<RepoFolderLinks>()(
  persist(
    (set) => ({
      links: {},
      link: (repo, folder) =>
        set((state) => ({ links: { ...state.links, [repo]: folder.replace(/(.)\/+$/, "$1") } })),
      unlink: (repo) =>
        set((state) => {
          const { [repo]: _removed, ...rest } = state.links;
          return { links: rest };
        }),
    }),
    { name: "cc3:github-repo-folders" },
  ),
);
