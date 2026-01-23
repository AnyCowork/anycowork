/**
 * End-to-End Test Setup for AnyCowork using WebDriver + tauri-driver
 * Official Tauri testing approach
 */

import { Builder, By, until, WebDriver } from 'selenium-webdriver';
import { Options } from 'selenium-webdriver/chrome';

export const E2E_CONFIG = {
  // Default model for testing
  DEFAULT_MODEL: 'gemini-3-pro-preview',
  DEFAULT_PROVIDER: 'gemini',

  // Test timeouts
  TIMEOUT: {
    SHORT: 5000,
    MEDIUM: 15000,
    LONG: 60000,
    AGENT_RESPONSE: 120000, // 2 minutes for agent responses
  },

  // Test agent configuration
  TEST_AGENT: {
    name: 'E2E Test Agent',
    description: 'Agent for end-to-end testing',
    personality: 'concise',
    tone: 'professional',
    systemPrompt: 'You are a test assistant. Provide brief, direct answers for testing purposes.',
  },

  // tauri-driver settings
  TAURI_DRIVER: {
    host: 'localhost',
    port: 4444,
  },
};

/**
 * Create WebDriver instance for Tauri app
 */
export async function createTauriDriver(): Promise<WebDriver> {
  const options = new Options();

  // tauri-driver specific capabilities
  options.setChromeBinaryPath('path/to/your/app'); // Will be set during build
  options.addArguments('--disable-blink-features=AutomationControlled');

  const driver = await new Builder()
    .forBrowser('chrome')
    .usingServer(`http://${E2E_CONFIG.TAURI_DRIVER.host}:${E2E_CONFIG.TAURI_DRIVER.port}`)
    .setChromeOptions(options)
    .build();

  // Set implicit wait
  await driver.manage().setTimeouts({ implicit: E2E_CONFIG.TIMEOUT.MEDIUM });

  return driver;
}

/**
 * Helper to wait for agent response
 */
export async function waitForAgentResponse(
  driver: WebDriver,
  timeout = E2E_CONFIG.TIMEOUT.AGENT_RESPONSE
): Promise<void> {
  // Wait for loading indicator to disappear
  try {
    const loadingElement = await driver.findElement(By.css('[data-testid="agent-loading"]'));
    await driver.wait(until.stalenessOf(loadingElement), timeout);
  } catch {
    // Loading indicator might not appear for fast responses
  }

  // Wait for response message to appear
  await driver.wait(
    until.elementLocated(By.css('[data-testid="assistant-message"]')),
    timeout
  );
}

/**
 * Helper to send message and wait for response
 */
export async function sendMessageAndWait(driver: WebDriver, message: string): Promise<void> {
  // Find input field
  const input = await driver.findElement(By.css('textarea[data-testid="chat-input"]'));

  // Clear and type message
  await input.clear();
  await input.sendKeys(message);

  // Click send button
  const sendButton = await driver.findElement(By.css('button[data-testid="send-button"]'));
  await sendButton.click();

  // Wait for response
  await waitForAgentResponse(driver);
}

/**
 * Helper to create test agent with Gemini 3 Pro
 */
export async function createTestAgent(driver: WebDriver): Promise<void> {
  // Navigate to agents page
  const agentsLink = await driver.findElement(By.css('a[href="/agents"]'));
  await agentsLink.click();

  // Wait for page load
  await driver.wait(until.urlContains('/agents'), E2E_CONFIG.TIMEOUT.MEDIUM);

  // Click create agent
  const createButton = await driver.findElement(By.css('button[data-testid="create-agent-button"]'));
  await createButton.click();

  // Fill agent details
  const nameInput = await driver.findElement(By.css('input[name="name"]'));
  await nameInput.sendKeys(E2E_CONFIG.TEST_AGENT.name);

  const descInput = await driver.findElement(By.css('textarea[name="description"]'));
  await descInput.sendKeys(E2E_CONFIG.TEST_AGENT.description);

  const promptInput = await driver.findElement(By.css('textarea[name="system_prompt"]'));
  await promptInput.sendKeys(E2E_CONFIG.TEST_AGENT.systemPrompt);

  // Select Gemini 3 Pro model
  const modelSelect = await driver.findElement(By.css('select[name="ai_model"]'));
  await modelSelect.sendKeys(E2E_CONFIG.DEFAULT_MODEL);

  // Save agent
  const submitButton = await driver.findElement(By.css('button[type="submit"]'));
  await submitButton.click();

  // Wait for creation
  await driver.wait(
    until.elementLocated(By.xpath(`//*[contains(text(), "${E2E_CONFIG.TEST_AGENT.name}")]`)),
    E2E_CONFIG.TIMEOUT.MEDIUM
  );
}

/**
 * Helper to start new chat session
 */
export async function startNewChat(driver: WebDriver): Promise<void> {
  // Navigate to chat
  const chatLink = await driver.findElement(By.css('a[href="/chat"]'));
  await chatLink.click();

  // Wait for chat page
  await driver.wait(until.urlContains('/chat'), E2E_CONFIG.TIMEOUT.MEDIUM);

  // Click new chat if needed
  try {
    const newChatButton = await driver.findElement(By.css('button[data-testid="new-chat-button"]'));
    if (await newChatButton.isDisplayed()) {
      await newChatButton.click();
    }
  } catch {
    // Button might not be visible
  }
}

/**
 * Verify agent is using Gemini 3 Pro
 */
export async function verifyGeminiModel(driver: WebDriver): Promise<void> {
  const modelSelector = await driver.findElement(By.css('[data-testid="model-selector"]'));
  const modelValue = await modelSelector.getText();

  if (!modelValue.includes('Gemini 3 Pro')) {
    throw new Error(`Expected Gemini 3 Pro but got: ${modelValue}`);
  }
}

/**
 * Take screenshot on failure
 */
export async function takeScreenshot(driver: WebDriver, name: string): Promise<void> {
  const screenshot = await driver.takeScreenshot();
  const fs = await import('fs');
  fs.writeFileSync(`./test-results/${name}.png`, screenshot, 'base64');
}

/**
 * Clean up driver
 */
export async function cleanupDriver(driver: WebDriver): Promise<void> {
  try {
    await driver.quit();
  } catch (error) {
    console.error('Error closing driver:', error);
  }
}
