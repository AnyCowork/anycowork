/**
 * End-to-End Chat Workflow Tests using WebDriver + tauri-driver
 * Tests complete user journey from UI to backend responses
 */

import { WebDriver } from 'selenium-webdriver';
import { describe, beforeEach, afterEach, test, expect } from '@jest/globals';
import {
  E2E_CONFIG,
  createTauriDriver,
  createTestAgent,
  startNewChat,
  sendMessageAndWait,
  verifyGeminiModel,
  cleanupDriver,
  takeScreenshot,
} from './setup';

describe('Chat Workflow E2E Tests', () => {
  let driver: WebDriver;

  beforeEach(async () => {
    driver = await createTauriDriver();

    // Wait for app to load
    await driver.sleep(2000);
  }, 30000); // 30 second timeout for setup

  afterEach(async () => {
    await cleanupDriver(driver);
  });

  test('E2E-001: Create agent and start chat session', async () => {
    try {
      // Create test agent with Gemini 3 Pro
      await createTestAgent(driver);

      // Start new chat
      await startNewChat(driver);

      // Verify chat interface loaded
      const chatInput = await driver.findElement({ css: 'textarea[data-testid="chat-input"]' });
      expect(await chatInput.isDisplayed()).toBe(true);

      // Verify Gemini 3 Pro is selected
      await verifyGeminiModel(driver);
    } catch (error) {
      await takeScreenshot(driver, 'E2E-001-failure');
      throw error;
    }
  }, E2E_CONFIG.TIMEOUT.LONG);

  test('E2E-002: Send simple message and receive response', async () => {
    try {
      // Start chat
      await startNewChat(driver);

      // Send message
      await sendMessageAndWait(driver, 'Hello, can you help me?');

      // Verify response appeared
      const assistantMessages = await driver.findElements({
        css: '[data-testid="assistant-message"]',
      });

      expect(assistantMessages.length).toBeGreaterThan(0);

      // Verify response has content
      const messageText = await assistantMessages[0].getText();
      expect(messageText).toBeTruthy();
      expect(messageText.length).toBeGreaterThan(0);
    } catch (error) {
      await takeScreenshot(driver, 'E2E-002-failure');
      throw error;
    }
  }, E2E_CONFIG.TIMEOUT.AGENT_RESPONSE + 10000);

  test('E2E-003: File operation workflow - list files', async () => {
    try {
      await startNewChat(driver);

      // Request file listing
      await sendMessageAndWait(driver, 'List files in the current directory');

      // Verify response
      const response = await driver.findElement({
        css: '[data-testid="assistant-message"]:last-child',
      });
      const responseText = await response.getText();

      // Should mention files or directories
      const hasFileContent = /file|directory|folder|\.tsx|\.ts|\.json/i.test(responseText);
      expect(hasFileContent).toBe(true);
    } catch (error) {
      await takeScreenshot(driver, 'E2E-003-failure');
      throw error;
    }
  }, E2E_CONFIG.TIMEOUT.AGENT_RESPONSE + 10000);

  test('E2E-004: Multi-turn conversation with context', async () => {
    try {
      await startNewChat(driver);

      // First message
      await sendMessageAndWait(driver, 'What files are in the src directory?');

      // Wait a moment
      await driver.sleep(1000);

      // Follow-up message (tests context retention)
      await sendMessageAndWait(driver, 'Can you count how many files you found?');

      // Verify both responses exist
      const messages = await driver.findElements({
        css: '[data-testid="assistant-message"]',
      });

      expect(messages.length).toBe(2);
    } catch (error) {
      await takeScreenshot(driver, 'E2E-004-failure');
      throw error;
    }
  }, E2E_CONFIG.TIMEOUT.AGENT_RESPONSE * 2 + 10000);

  test('E2E-005: Error handling - invalid request', async () => {
    try {
      await startNewChat(driver);

      // Send unclear message
      await sendMessageAndWait(driver, 'Fix it');

      // Agent should ask for clarification
      const response = await driver.findElement({
        css: '[data-testid="assistant-message"]:last-child',
      });
      const responseText = await response.getText();

      const hasClarificationRequest = /clarif|what|which|specify|could you/i.test(responseText);
      expect(hasClarificationRequest).toBe(true);
    } catch (error) {
      await takeScreenshot(driver, 'E2E-005-failure');
      throw error;
    }
  }, E2E_CONFIG.TIMEOUT.AGENT_RESPONSE + 10000);

  test('E2E-006: Session management - new chat creates new session', async () => {
    try {
      await startNewChat(driver);

      // Send first message
      await sendMessageAndWait(driver, 'First session message');

      // Get session ID from URL
      const firstSessionUrl = await driver.getCurrentUrl();
      const firstSessionId = firstSessionUrl.split('/').pop();

      // Create new chat
      const newChatButton = await driver.findElement({
        css: 'button[data-testid="new-chat-button"]',
      });
      await newChatButton.click();

      // Wait for new URL
      await driver.sleep(1000);

      // Get new session ID
      const newSessionUrl = await driver.getCurrentUrl();
      const newSessionId = newSessionUrl.split('/').pop();

      // Verify different sessions
      expect(firstSessionId).not.toBe(newSessionId);

      // Verify new chat is empty
      const messages = await driver.findElements({
        css: '[data-testid="assistant-message"]',
      });

      expect(messages.length).toBe(0);
    } catch (error) {
      await takeScreenshot(driver, 'E2E-006-failure');
      throw error;
    }
  }, E2E_CONFIG.TIMEOUT.AGENT_RESPONSE + 10000);
});
