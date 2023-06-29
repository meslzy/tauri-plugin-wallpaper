import React from "react";

import { invoke } from "@tauri-apps/api/tauri";

import { Button, Center, Flex, Text } from "@mantine/core";

const App = () => {
  const handleClick = () => {
    return invoke( "my_custom_command" ).then( res => {
      console.log( res );
    });
  };

  return (
    <Center h={ "100%" }>
      <Flex align={ "center" } direction={ "column" }>
        <Text>Tauri </Text>
        <Button onClick={ handleClick }>Invoke my command</Button>
      </Flex>
    </Center>
  );
};

export default App;
