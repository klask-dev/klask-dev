import React from "react";
import { Outlet } from "react-router-dom";
import { Navbar } from "./Navbar";
import { Sidebar } from "./Sidebar";

export const AppLayout: React.FC = () => {
    return (
        <div className="min-h-screen bg-gray-50">
            <Navbar />

            <div className="flex flex-col lg:flex-row">
                {/* Sidebar - Always visible on desktop, shown first on mobile */}
                <div className="lg:fixed lg:inset-y-0 lg:flex lg:w-72 lg:flex-col lg:pt-16">
                    <Sidebar />
                </div>

                {/* Main content */}
                <main className="flex-1 lg:pl-72">
                    <div className="px-4 pt-20 pb-8 sm:px-6 lg:px-8">
                        <Outlet />
                    </div>
                </main>
            </div>
        </div>
    );
};
