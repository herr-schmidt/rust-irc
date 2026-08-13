import { JSX, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { User } from "./utils/const";
import "overlayscrollbars/overlayscrollbars.css";
import { listen } from "@tauri-apps/api/event";

function App() {
  const [users, setUsers] = useState<User[]>();
  const [usersItems, setUsersItems] = useState<JSX.Element[]>();
  const [userInput, setUserInput] = useState<string>("");

  useEffect(() => {
    listen<User[]>("new-users", (event) => {
      console.log(event.payload);
      setUsers(event.payload);
    });
  }, []);

  useEffect(() => {
    const newUsersItems: JSX.Element[] = [];
    users?.forEach((user: User, index: number) =>
      newUsersItems.push(
        <li
          className="pl-2 pr-2 cursor-pointer select-none border-l-[3px] border-gray-900 hover:border-blue-400"
          key={index}
        >
          {user.nickname}
        </li>,
      ),
    );
    setUsersItems(newUsersItems);
  }, [users]);

  return (
    <main className="flex flex-row w-full h-full">
      <div className="flex flex-col w-full h-min-fit bg-gray-800 p-2 gap-y-2">
        <div className="flex rounded-sm border border-gray-600 w-min-fit h-full"></div>
        <input
          className="flex w-full border border-gray-600 rounded-sm focus:outline-none p-1"
          type="text"
          value={userInput}
          onChange={(event) => setUserInput(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              invoke("process_user_input_line", { userInputLine: userInput });
              setUserInput("");
            }
          }}
        ></input>
      </div>
      <div className="flex flex-col w-[16rem] bg-gray-900 overflow-y-scroll text-sm font-bold text-emerald-600">
        <ul>{usersItems}</ul>
      </div>
    </main>
  );
}

export default App;
