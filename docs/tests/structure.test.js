const fs = require("fs");
const path = require("path");
const { EOL } = require("os");

describe("Is compatible with the docs hub", () => {
  const srcFolderPath = path.join(__dirname, "../src");
  const subfolders = fs
    .readdirSync(srcFolderPath)
    .filter((item) =>
      fs.statSync(path.join(srcFolderPath, item)).isDirectory()
    );
  const summaryFilePath = path.join(srcFolderPath, "SUMMARY.md");
  const summaryContent = fs.readFileSync(summaryFilePath, "utf-8");
  const splitSummary = summaryContent.split(EOL);

  it("should have an index file at the root", () => {
    const indexPath = path.join(srcFolderPath, "index.md");
    expect(fs.existsSync(indexPath)).toBe(true);
  });


  it("should not have nested subfolders", () => {
    const nestedSubfolders = subfolders.filter((subfolder) =>
      fs
        .readdirSync(path.join(srcFolderPath, subfolder))
        .some((item) =>
          fs.statSync(path.join(srcFolderPath, subfolder, item)).isDirectory()
        )
    );
    expect(nestedSubfolders).toHaveLength(0);
  });

  it("should not have nested subfolders in the SUMMARY file", () => {
    const nestedSubfolders = splitSummary.filter((line) => {
     return line.startsWith("    -");
  });
    expect(nestedSubfolders).toHaveLength(0);
  });

  it("should not have unused files", () => {
    const fileNames = fs.readdirSync(srcFolderPath);
    fileNames.forEach((file) => {
      // check if each file can be found in the SUMMARY
      if(file !== "SUMMARY.md"){
        expect(summaryContent.includes(file)).toBe(true);
      }
    })
    subfolders.forEach((folder) => {
      const folderPath = path.join(srcFolderPath, folder);
      const subfolderNames = fs.readdirSync(folderPath);
      subfolderNames.forEach((subFile) => {
        expect(summaryContent.includes(subFile)).toBe(true);
      })
    })
  });

  it("should have a folder structure that matches the SUMMARY.md order", () => {
    const order = processSummary(splitSummary);

    Object.keys(order).forEach((key) => {
      const menuOrder = order[key];
      if (key === "menu") {
        // check if each line in the menu corresponds to
        // an existing top-level file or the name of the folder
        menuOrder.forEach((item) => {
          let itemPath = path.join(srcFolderPath, item);
          if (fs.existsSync(itemPath)) {
            expect(fs.statSync(itemPath).isDirectory()).toBe(true);
          } else {
            itemPath = `${itemPath}.md`;
            expect(fs.existsSync(itemPath)).toBe(true);
          }
        });
      } else {
        // check if item exists in the right folder
        const possibleSeparators = ["-", "_"];
        menuOrder.forEach((item) => {
          let fileExists = false;
          for (const separator of possibleSeparators) {
            const newItem = item.replace(/[-_]/g, separator);
            let itemPath = path.join(srcFolderPath, `${key}/${newItem}.md`);
            if (fs.existsSync(itemPath)) {
              fileExists = true;
              break;
            }
          }
          expect(fileExists).toBe(true);
        });
      }
    });
  });
});

function processSummary(lines) {
  const order = { menu: [] };
  let currentCategory;
  lines.forEach((line) => {
    const paths = line.split("/");
    const newPaths = paths[0].split("(");
    const thisCat = currentCategory;
    if (line.includes(".md")) {
      if (line[0] === "-") {
        // handle top-level items
        if (paths.length > 2) {
          currentCategory = paths[paths.length - 2];
        } else if (
          paths[paths.length - 1].includes("index.md") ||
          newPaths[newPaths.length - 1].endsWith(".md)")
        ) {
          currentCategory = newPaths[newPaths.length - 1];
        } else {
          currentCategory = paths[paths.length - 1];
        }
        const final = currentCategory.replace(".md)", "");
        if (thisCat === final) {
          const fileName = paths[paths.length - 1].replace(".md)", "");
          if (!order[currentCategory]) order[currentCategory] = [];
          order[currentCategory].push(fileName);
        } else if (final !== "index") {
          order.menu.push(final);
        }
      } else if (currentCategory) {
        // handle sub-paths
        const fileName = paths[paths.length - 1].replace(".md)", "");
        if (!order[currentCategory]) order[currentCategory] = [];
        if (fileName !== "index") order[currentCategory].push(fileName);
      }
    }
  });
  return order;
}
