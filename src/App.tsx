import { HashRouter, Route, Routes } from "react-router-dom";
import Layout from "@/components/Layout";
import HistoryPage from "@/pages/History";
import Home from "@/pages/Home";
import SettingsPage from "@/pages/Settings";

export default function App() {
  return (
    <HashRouter>
      <Layout>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/history" element={<HistoryPage />} />
        </Routes>
      </Layout>
    </HashRouter>
  );
}
