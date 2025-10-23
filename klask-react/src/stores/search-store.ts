import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { SearchFilters, SearchQuery } from "../types";

interface SearchState {
    // Current search state
    query: string;
    activeFilters: {
        projects: string[];
        versions: string[];
        extensions: string[];
    };

    // Pagination
    page: number;
    size: number;
    sort: string;

    // Available filters with counts
    availableFilters: SearchFilters;

    // Search history
    searchHistory: string[];

    // UI state
    sidebarOpen: boolean;

    // Actions
    setQuery: (query: string) => void;
    setFilters: (filters: Partial<SearchState["activeFilters"]>) => void;
    addFilter: (
        type: keyof SearchState["activeFilters"],
        value: string,
    ) => void;
    removeFilter: (
        type: keyof SearchState["activeFilters"],
        value: string,
    ) => void;
    clearFilters: () => void;

    setPagination: (page?: number, size?: number, sort?: string) => void;
    resetPagination: () => void;

    setAvailableFilters: (filters: SearchFilters) => void;

    addToHistory: (query: string) => void;
    clearHistory: () => void;

    toggleSidebar: () => void;
    setSidebarOpen: (open: boolean) => void;

    // Computed values
    hasActiveFilters: () => boolean;
    getSearchQuery: () => SearchQuery;
    getCurrentFilters: () => SearchState["activeFilters"];
}

const DEFAULT_PAGE_SIZE = 20;
const MAX_HISTORY_ITEMS = 10;

// Detect if we're on mobile to set initial sidebar state
const isMobile = typeof window !== 'undefined' && window.innerWidth < 1024;

const initialState = {
    query: "",
    activeFilters: {
        projects: [],
        versions: [],
        extensions: [],
    },
    page: 1,
    size: DEFAULT_PAGE_SIZE,
    sort: "relevance",
    availableFilters: {
        projects: [],
        versions: [],
        extensions: [],
    },
    searchHistory: [],
    sidebarOpen: !isMobile, // Closed on mobile, open on desktop
};

export const useSearchStore = create<SearchState>()(
    persist(
        (set, get) => ({
            ...initialState,

            setQuery: (query) => {
                set({ query, page: 1 }); // Reset to first page on new search

                // Add to history if it's a meaningful search
                if (query.trim().length > 0) {
                    get().addToHistory(query.trim());
                }
            },

            setFilters: (filters) => {
                set((state) => ({
                    activeFilters: { ...state.activeFilters, ...filters },
                    page: 1, // Reset to first page when filters change
                }));
            },

            addFilter: (type, value) => {
                set((state) => {
                    const currentFilters = state.activeFilters[type] || [];
                    if (!currentFilters.includes(value)) {
                        return {
                            activeFilters: {
                                ...state.activeFilters,
                                [type]: [...currentFilters, value],
                            },
                            page: 1, // Reset pagination
                        };
                    }
                    return state;
                });
            },

            removeFilter: (type, value) => {
                set((state) => ({
                    activeFilters: {
                        ...state.activeFilters,
                        [type]: (state.activeFilters[type] || []).filter(
                            (item) => item !== value,
                        ),
                    },
                    page: 1, // Reset pagination
                }));
            },

            clearFilters: () => {
                set({
                    activeFilters: {
                        projects: [],
                        versions: [],
                        extensions: [],
                    },
                    page: 1,
                });
            },

            setPagination: (page, size, sort) => {
                set((state) => ({
                    page: page ?? state.page,
                    size: size ?? state.size,
                    sort: sort ?? state.sort,
                }));
            },

            resetPagination: () => {
                set({ page: 1 });
            },

            setAvailableFilters: (filters) => {
                set({ availableFilters: filters });
            },

            addToHistory: (query) => {
                set((state) => {
                    const newHistory = [
                        query,
                        ...state.searchHistory.filter((item) => item !== query),
                    ].slice(0, MAX_HISTORY_ITEMS);

                    return { searchHistory: newHistory };
                });
            },

            clearHistory: () => {
                set({ searchHistory: [] });
            },

            toggleSidebar: () => {
                set((state) => ({ sidebarOpen: !state.sidebarOpen }));
            },

            setSidebarOpen: (open) => {
                set({ sidebarOpen: open });
            },

            hasActiveFilters: () => {
                const { activeFilters } = get();
                return (
                    activeFilters.projects.length > 0 ||
                    activeFilters.versions.length > 0 ||
                    activeFilters.extensions.length > 0
                );
            },

            getSearchQuery: () => {
                const state = get();
                return {
                    query: state.query,
                    project:
                        state.activeFilters.projects.join(",") || undefined,
                    version:
                        state.activeFilters.versions.join(",") || undefined,
                    extension:
                        state.activeFilters.extensions.join(",") || undefined,
                    maxResults: state.size,
                };
            },

            getCurrentFilters: () => {
                return get().activeFilters;
            },
        }),
        {
            name: "klask-search",
            partialize: (state) => ({
                query: state.query,
                activeFilters: state.activeFilters,
                size: state.size,
                sort: state.sort,
                searchHistory: state.searchHistory,
                sidebarOpen: state.sidebarOpen,
            }),
        },
    ),
);

// Selectors for convenient access
export const searchSelectors = {
    query: () => useSearchStore((state) => state.query),
    activeFilters: () => useSearchStore((state) => state.activeFilters),
    availableFilters: () => useSearchStore((state) => state.availableFilters),
    pagination: () =>
        useSearchStore((state) => ({
            page: state.page,
            size: state.size,
            sort: state.sort,
        })),
    searchHistory: () => useSearchStore((state) => state.searchHistory),
    sidebarOpen: () => useSearchStore((state) => state.sidebarOpen),
    hasActiveFilters: () => useSearchStore((state) => state.hasActiveFilters()),
    searchQuery: () => useSearchStore((state) => state.getSearchQuery()),
};

// Utility functions
export const searchUtils = {
    // Build URL search params for sharing/bookmarking
    buildSearchParams: (state: SearchState): URLSearchParams => {
        const params = new URLSearchParams();

        if (state.query) params.set("q", state.query);
        if (state.activeFilters.projects.length > 0) {
            params.set("projects", state.activeFilters.projects.join(","));
        }
        if (state.activeFilters.versions.length > 0) {
            params.set("versions", state.activeFilters.versions.join(","));
        }
        if (state.activeFilters.extensions.length > 0) {
            params.set("extensions", state.activeFilters.extensions.join(","));
        }
        if (state.page > 1) params.set("page", state.page.toString());
        if (state.size !== DEFAULT_PAGE_SIZE)
            params.set("size", state.size.toString());
        if (state.sort !== "relevance") params.set("sort", state.sort);

        return params;
    },

    // Parse URL search params to restore state
    parseSearchParams: (searchParams: URLSearchParams) => {
        return {
            query: searchParams.get("q") || "",
            activeFilters: {
                projects:
                    searchParams.get("projects")?.split(",").filter(Boolean) ||
                    [],
                versions:
                    searchParams.get("versions")?.split(",").filter(Boolean) ||
                    [],
                extensions:
                    searchParams
                        .get("extensions")
                        ?.split(",")
                        .filter(Boolean) || [],
            },
            page: parseInt(searchParams.get("page") || "1", 10),
            size: parseInt(
                searchParams.get("size") || DEFAULT_PAGE_SIZE.toString(),
                10,
            ),
            sort: searchParams.get("sort") || "relevance",
        };
    },

    // Reset search state
    resetSearch: () => {
        useSearchStore.setState(initialState);
    },
};
