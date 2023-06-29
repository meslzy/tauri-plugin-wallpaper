import React from "react";

import * as event from "@tauri-apps/api/event";
import { appWindow } from "@tauri-apps/api/window";

import type { MantineThemeOverride } from "@mantine/core";
import { MantineProvider } from "@mantine/core";

interface Props {
  children: React.ReactNode;
}

const Mantine = ( props: Props ) => {
  const [ colorScheme, setColorScheme ] = React.useState<MantineThemeOverride["colorScheme"]>(() => {
    return "light";
  });

  const theme: MantineThemeOverride = {
    colorScheme,
    loader: "bars",
    fontFamily: "inherit",
    globalStyles: () => {
      return {
        "*, *::before, *::after": {
          boxSizing: "border-box",
          userSelect: "none",
          margin: 0,
          padding: 0,
        },
        "img, svg, a": {
          WebkitUserDrag: "none",
        },
        "body": {
          height: "100vh",
        },
        "#app": {
          all: "inherit",
        },
        "a[href]": {
          color: "inherit",
        },
      }; 
    },
  };

  React.useEffect(() => {
    appWindow.theme().then( (theme) => {
      if ( theme !== null ) {
        setColorScheme( theme );
      }
    });
  }, []);

  React.useEffect(() => {
    event.listen( "tauri://theme-changed", (event) => {
      console.log( event.payload );
    });
  }, []);

  return (
    <MantineProvider theme={ theme } withGlobalStyles={ true } withNormalizeCSS={ true }>
      { props.children }
    </MantineProvider>
  );
};

export default Mantine;