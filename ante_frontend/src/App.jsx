import { BrowserRouter, Routes, Route } from "react-router-dom";
import LandPage from "./components/LandingPage/LandPage";
function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<LandPage />}></Route>
      </Routes>
    </BrowserRouter>
  );
}

export default App;
