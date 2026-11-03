(function () {
  try {
    var choice = localStorage.getItem("codemd:theme") || "dark";
    var light =
      choice === "light" ||
      (choice === "system" && window.matchMedia("(prefers-color-scheme: light)").matches);
    document.documentElement.dataset.theme = light ? "light" : "dark";
  } catch (error) {
    document.documentElement.dataset.theme = "dark";
  }
})();
