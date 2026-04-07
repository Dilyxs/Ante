import { BrowserRouter, Routes, Route } from "react-router-dom";
import LandingLayout from "./components/LandingPage/LandingLayout";
import MainPage from "./components/LandingPage/MainPage";
import HowItWorksPage from "./components/LandingPage/HowItWorksPage";
import NotFoundPage from "./components/LandingPage/NotFoundPage";
import Dashboard from "./components/mainapp/Dashboard";

function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<LandingLayout />}>
          <Route index element={<MainPage />} />
          <Route path="howitworks" element={<HowItWorksPage />} />
          <Route path="*" element={<NotFoundPage />} />
        </Route>
        <Route path="dashboard" element={<Dashboard />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
