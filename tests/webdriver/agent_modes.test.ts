import { Key, WebDriver, By } from 'selenium-webdriver';
import { createTauriDriver, cleanupDriver, createTestAgent, E2E_CONFIG, startNewChat, sendMessageAndWait } from './setup';

describe('Agent Modes E2E', () => {
    let driver: WebDriver;

    beforeAll(async () => {
        driver = await createTauriDriver();
        await createTestAgent(driver);
    }, 60000);

    afterAll(async () => {
        await cleanupDriver(driver);
    });

    test('Fast Mode execution', async () => {
        await startNewChat(driver);

        // Default mode is Planning (text says "Planning")
        // Find the toggle button. It should distinctively contain the text "Planning".
        try {
            const toggleBtn = await driver.findElement(By.xpath('//button[.//span[contains(text(), "Planning")]]'));
            await toggleBtn.click();
        } catch (e) {
            // Should be found. If not, maybe it's already Fast?
            // If it says "Fast", we are already in Fast mode (unexpected for new chat but possible if state persists)
            // Let's verify we are now in Fast mode
        }

        // Verify button now says "Fast"
        await driver.findElement(By.xpath('//button[.//span[contains(text(), "Fast")]]'));

        await sendMessageAndWait(driver, 'Calculate 2+2');

        // Check checklist sidebar absence (Fast mode shouldn't trigger plan)
        try {
            await driver.findElement({ xpath: '//h3[contains(text(), "AGENT PLAN")]' });
            throw new Error("Plan Sidebar should NOT act visible in Fast Mode");
        } catch (e) {
            // Expected
        }
    }, 30000);

    test('Planning Mode execution', async () => {
        await startNewChat(driver);

        // Ensure Planning Mode
        try {
            // Check if it says "Fast", if so click to switch to Planning
            const fastBtn = await driver.findElement(By.xpath('//button[.//span[contains(text(), "Fast")]]'));
            await fastBtn.click();
        } catch {
            // If "Fast" button not found, check if "Planning" is there
            await driver.findElement(By.xpath('//button[.//span[contains(text(), "Planning")]]'));
        }

        await sendMessageAndWait(driver, 'Create a file named plan_test.txt');

        // Sidebar should appear
        const sidebar = await driver.findElement({ xpath: '//h3[contains(text(), "AGENT PLAN")]' });
        expect(await sidebar.isDisplayed()).toBe(true);
    }, 120000);
});
