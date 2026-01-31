/**
 * End-to-End Agent Management Tests
 * Tests agent creation, configuration, and management workflows
 */

import { WebDriver, By, until } from 'selenium-webdriver';
import { describe, beforeEach, afterEach, test, expect } from '@jest/globals';
import {
    E2E_CONFIG,
    createTauriDriver,
    cleanupDriver,
    takeScreenshot,
} from './setup';

describe('Agent Management E2E Tests', () => {
    let driver: WebDriver;

    beforeEach(async () => {
        driver = await createTauriDriver();
        // Wait for app to load
        await driver.sleep(2000);
    }, 30000);

    afterEach(async () => {
        await cleanupDriver(driver);
    });

    test('E2E-AGENT-001: Create new agent with Gemini 3 Pro', async () => {
        try {
            // Navigate to agents page
            const agentsLink = await driver.findElement(By.css('a[href="/agents"]'));
            await agentsLink.click();
            await driver.wait(until.urlContains('/agents'), E2E_CONFIG.TIMEOUT.MEDIUM);

            // Click create agent button
            const createButton = await driver.findElement(By.css('button[data-testid="create-agent-button"]'));
            await createButton.click();

            // Fill agent form
            const nameInput = await driver.findElement(By.css('input[name="name"]'));
            await nameInput.sendKeys('Test Agent Gemini');

            const descInput = await driver.findElement(By.css('textarea[name="description"]'));
            await descInput.sendKeys('Test agent using Gemini 3 Pro');

            // Select Gemini 3 Pro
            const modelSelect = await driver.findElement(By.css('select[name="ai_model"]'));
            await modelSelect.click();
            const option = await driver.findElement(By.css('select[name="ai_model"] option[value="gemini-3-flash-preview"]'));
            await option.click();

            // Verify selection (getting value from select usually returns the value of selected option)
            const selectedModel = await modelSelect.getAttribute('value');
            expect(selectedModel).toBe('gemini-3-flash-preview');

            // Set personality
            const personalitySelect = await driver.findElement(By.css('select[name="personality"]'));
            await personalitySelect.sendKeys('concise');

            // Fill system prompt
            const promptInput = await driver.findElement(By.css('textarea[name="system_prompt"]'));
            await promptInput.sendKeys('You are a helpful test assistant.');

            // Save agent
            const submitButton = await driver.findElement(By.css('button[type="submit"]'));
            await submitButton.click();

            // Verify agent created
            await driver.wait(
                until.elementLocated(By.xpath('//*[contains(text(), "Test Agent Gemini")]')),
                E2E_CONFIG.TIMEOUT.MEDIUM
            );
        } catch (error) {
            await takeScreenshot(driver, 'E2E-AGENT-001-failure');
            throw error;
        }
    }, E2E_CONFIG.TIMEOUT.LONG);

    test('E2E-AGENT-002: View agent list', async () => {
        try {
            // Navigate to agents page
            const agentsLink = await driver.findElement(By.css('a[href="/agents"]'));
            await agentsLink.click();
            await driver.wait(until.urlContains('/agents'), E2E_CONFIG.TIMEOUT.MEDIUM);

            // Verify agents list loads
            const agentsList = await driver.findElement(By.css('[data-testid="agents-list"]'));
            expect(await agentsList.isDisplayed()).toBe(true);

            // Should have at least default agent
            const agentCards = await driver.findElements(By.css('[data-testid^="agent-card"]'));
            expect(agentCards.length).toBeGreaterThan(0);
        } catch (error) {
            await takeScreenshot(driver, 'E2E-AGENT-002-failure');
            throw error;
        }
    }, E2E_CONFIG.TIMEOUT.MEDIUM);

    test('E2E-AGENT-003: Edit existing agent', async () => {
        try {
            // Navigate to agents page
            const agentsLink = await driver.findElement(By.css('a[href="/agents"]'));
            await agentsLink.click();
            await driver.wait(until.urlContains('/agents'), E2E_CONFIG.TIMEOUT.MEDIUM);

            // Click first agent's edit button - looking for specific nesting or just first edit button
            const editButtons = await driver.findElements(By.css('[data-testid="edit-agent-button"]'));
            if (editButtons.length === 0) throw new Error('No edit buttons found');
            await editButtons[0].click();

            // Wait for edit form
            const nameInput = await driver.wait(
                until.elementLocated(By.css('input[name="name"]')),
                E2E_CONFIG.TIMEOUT.MEDIUM
            );

            // Modify name
            const originalName = await nameInput.getAttribute('value');
            const newName = originalName + ' (Edited)';
            await nameInput.clear();
            await nameInput.sendKeys(newName);

            // Save changes
            const saveButton = await driver.findElement(By.css('button[type="submit"]'));
            await saveButton.click();

            // Verify update
            await driver.wait(
                until.elementLocated(By.xpath(`//*[contains(text(), "${newName}")]`)),
                E2E_CONFIG.TIMEOUT.MEDIUM
            );
        } catch (error) {
            await takeScreenshot(driver, 'E2E-AGENT-003-failure');
            throw error;
        }
    }, E2E_CONFIG.TIMEOUT.LONG);

    test('E2E-AGENT-004: Agent configuration - temperature setting', async () => {
        try {
            // Navigate and create
            const agentsLink = await driver.findElement(By.css('a[href="/agents"]'));
            await agentsLink.click();

            const createButton = await driver.findElement(By.css('button[data-testid="create-agent-button"]'));
            await createButton.click();

            // Set temperature
            const tempInput = await driver.findElement(By.css('input[name="temperature"]'));
            await tempInput.clear();
            await tempInput.sendKeys('0.9');

            const temperatureValue = await tempInput.getAttribute('value');
            expect(parseFloat(temperatureValue)).toBe(0.9);
        } catch (error) {
            await takeScreenshot(driver, 'E2E-AGENT-004-failure');
            throw error;
        }
    }, E2E_CONFIG.TIMEOUT.MEDIUM);

    test('E2E-AGENT-005: Agent with custom system prompt', async () => {
        try {
            const agentsLink = await driver.findElement(By.css('a[href="/agents"]'));
            await agentsLink.click();

            const createButton = await driver.findElement(By.css('button[data-testid="create-agent-button"]'));
            await createButton.click();

            const customPrompt = 'You are a specialized assistant for testing. Always respond with "TEST_RESPONSE" followed by your answer.';
            const agentName = 'Custom Prompt Agent';

            const nameInput = await driver.findElement(By.css('input[name="name"]'));
            await nameInput.sendKeys(agentName);

            const promptInput = await driver.findElement(By.css('textarea[name="system_prompt"]'));
            await promptInput.sendKeys(customPrompt);

            const modelSelect = await driver.findElement(By.css('select[name="ai_model"]'));
            await modelSelect.sendKeys(E2E_CONFIG.DEFAULT_MODEL);

            const submitButton = await driver.findElement(By.css('button[type="submit"]'));
            await submitButton.click();

            // Verify agent created with custom prompt
            await driver.wait(
                until.elementLocated(By.xpath(`//*[contains(text(), "${agentName}")]`)),
                E2E_CONFIG.TIMEOUT.MEDIUM
            );
        } catch (error) {
            await takeScreenshot(driver, 'E2E-AGENT-005-failure');
            throw error;
        }
    }, E2E_CONFIG.TIMEOUT.LONG);
});
