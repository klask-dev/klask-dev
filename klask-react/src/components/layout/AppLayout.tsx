import React from "react";
import { Outlet } from "react-router-dom";
import { Navbar } from "./Navbar";
import { Sidebar } from "./Sidebar";
import { ResizeHandle } from "./ResizeHandle";
import { useResizable } from "../../hooks/useResizable";

export const AppLayout: React.FC = () => {
    const { width, handleMouseDown } = useResizable({
        initialWidth: 288, // 72 * 4 (TailwindCSS w-72)
        minWidth: 200,
        maxWidth: 600,
        storageKey: 'sidebar-width',
    });

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-950">
            <Navbar />

            <div className="flex flex-col lg:flex-row">
                {/* Sidebar - Always visible on desktop, shown first on mobile */}
                <div
                    className="lg:fixed lg:inset-y-0 lg:flex lg:flex-col lg:pt-16 relative"
                    style={{ width: `${width}px` }}
                >
                    <Sidebar />
                    <ResizeHandle onMouseDown={handleMouseDown} />
                </div>

                {/* Main content */}
                <main className="flex-1 lg:pt-16" style={{ marginLeft: `${width}px` }}>
                    <div className="px-4 pt-20 pb-8 sm:px-6 lg:px-8">
                        <Outlet />
                    </div>
                </main>
            </div>
        </div>
    );
};
