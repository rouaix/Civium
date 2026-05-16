import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import type { RootStackParamList } from './src/types';

import OnboardingScreen from './src/screens/OnboardingScreen';
import PairingScreen    from './src/screens/PairingScreen';
import NetworksScreen   from './src/screens/NetworksScreen';
import MessagesScreen   from './src/screens/MessagesScreen';

const Stack = createNativeStackNavigator<RootStackParamList>();

export default function App() {
  return (
    <SafeAreaProvider>
      <NavigationContainer>
        <Stack.Navigator
          initialRouteName="Onboarding"
          screenOptions={{
            headerStyle:       { backgroundColor: '#fff' },
            headerTintColor:   '#4f46e5',
            headerTitleStyle:  { fontWeight: '700' },
            headerShadowVisible: true,
          }}
        >
          <Stack.Screen
            name="Onboarding"
            component={OnboardingScreen}
            options={{ headerShown: false }}
          />
          <Stack.Screen
            name="Pairing"
            component={PairingScreen}
            options={{ title: 'Jumeler un appareil', headerShown: false }}
          />
          <Stack.Screen
            name="Networks"
            component={NetworksScreen}
            options={{ title: 'Mes réseaux', headerBackVisible: false }}
          />
          <Stack.Screen
            name="Messages"
            component={MessagesScreen}
            options={({ route }) => ({ title: route.params.network.name })}
          />
        </Stack.Navigator>
      </NavigationContainer>
    </SafeAreaProvider>
  );
}
