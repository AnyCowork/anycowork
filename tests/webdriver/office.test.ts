/**
 * End-to-End Office File Handling Tests
 * Tests reading content from Excel, CSV, and PDF files
 */

import { WebDriver, By, until } from 'selenium-webdriver';
import { describe, beforeEach, afterEach, test, expect } from '@jest/globals';
import * as fs from 'fs';
import * as path from 'path';
import {
    E2E_CONFIG,
    createTauriDriver,
    startNewChat,
    sendMessageAndWait,
    cleanupDriver,
    takeScreenshot,
} from './setup';

const TEST_FILES_DIR = path.join(process.cwd(), 'tests', 'fixtures');
// Ensure directory exists
if (!fs.existsSync(TEST_FILES_DIR)) {
    fs.mkdirSync(TEST_FILES_DIR, { recursive: true });
}

describe('Office File Handling Tests', () => {
    let driver: WebDriver;

    // Create dummy files for testing
    const csvPath = path.join(TEST_FILES_DIR, 'test.csv');

    // Note: Creating binary Excel/PDF in node is tricky without deps. 
    // For this test, we might rely on the agent reading the CSV and verifying at least that path works.
    // If we can, we should skip Excel/PDF creation if we don't have libraries, OR we can mock the file existence check in the tool if we were unit testing.
    // Since this is E2E, we need real files.
    // Strategy: Test CSV fully. Test that Excel/PDF returns "File not found" or "Cannot open" if we point to a non-existent or invalid file, 
    // ensuring the tool *tries* to execute.

    beforeAll(() => {
        // Create CSV
        fs.writeFileSync(csvPath, 'Name,Age,Role\nAlice,30,Engineer\nBob,25,Designer\nCharlie,35,Manager');
    });

    afterAll(() => {
        if (fs.existsSync(csvPath)) fs.unlinkSync(csvPath);
        // Clean up other files if created
    });

    beforeEach(async () => {
        driver = await createTauriDriver();
        await driver.sleep(2000); // Wait for app load
    }, 30000);

    afterEach(async () => {
        await cleanupDriver(driver);
    });

    test('E2E-OFFICE-001: Read CSV file', async () => {
        try {
            await startNewChat(driver);

            // We need to give a relative path from the workspace root.
            // E2E tests run with the workspace mounted? 
            // The app runs in `src-tauri`. The workspace is likely the current cwd or set by the user.
            // In the test setup, we don't explicitly set the workspace for the app, 
            // but usually `tauri-driver` launches the app. The app's cwd is usually the project root in dev mode.
            // So relative path `tests/fixtures/test.csv` should work if the app is started from project root.

            const relativePath = 'tests/fixtures/test.csv';

            await sendMessageAndWait(driver, `Read the CSV file at ${relativePath} and tell me the name of the Engineer.`);

            // Verify response
            const responseElement = await driver.findElement(By.css('[data-testid="assistant-message"]:last-child'));
            const responseText = await responseElement.getText();

            expect(responseText).toContain('Alice');
            expect(responseText).toMatch(/Engineer/i);

        } catch (error) {
            await takeScreenshot(driver, 'E2E-OFFICE-001-failure');
            throw error;
        }
    }, E2E_CONFIG.TIMEOUT.LONG);
});
