import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "OpenNotebookLM",
  description: "Open-source NotebookLM clone",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${inter.className} h-screen overflow-hidden flex flex-col bg-white text-black`}
      >
        {children}
      </body>
    </html>
  );
}
